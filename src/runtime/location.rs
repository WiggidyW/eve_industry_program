use super::*;
use crate::api_data_rename as api_data;
use crate::config;
use crate::config::IndustrySlots;
use crate::config::Item;
use crate::config::ProductionLineExportKind;
use crate::industry_db;
use std::cell::Ref;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

fn deduplicate_locations<'iter, 'cfg, 'db, 'api>(
    locations: impl Iterator<Item = &'iter Location<'cfg, 'db, 'api>>,
) -> Vec<&'iter Location<'cfg, 'db, 'api>> {
    let mut seen = HashMap::<u64, bool>::new();
    locations
        .filter(move |location| {
            let id = location.id();
            if seen.contains_key(&id) {
                false
            } else {
                seen.insert(id, true);
                true
            }
        })
        .collect()
}

fn assets_target<'cfg, 'db, 'api>(
    locations: &[Rc<Location<'cfg, 'db, 'api>>],
) -> HashMap<u64, HashMap<Item, i64>> {
    let mut assets_target = HashMap::<u64, HashMap<Item, i64>>::new();
    for location in locations {
        for production_line in location.production_lines().iter_all() {
            // add product to locations along export pipe
            // only do this for products, as intermediates will be added by the import
            if production_line.export_kind()
                == ProductionLineExportKind::Product
            {
                let item = production_line.product();
                let num_buildable = production_line.num_buildable();
                for location in production_line.export_pipe().locations() {
                    *assets_target
                        .entry(location.id())
                        .or_insert_with(HashMap::new)
                        .entry(item)
                        .or_insert(0) += num_buildable;
                }
            }

            // add materials to locations along import pipes
            let max_num_builds = production_line.max_num_builds();
            let market_locations = deduplicate_locations(
                production_line
                    .import_src_market_pipes()
                    .map(|p| p.dst_locations())
                    .flatten(),
            );

            for (item, quantity) in production_line.minerals_i64() {
                // check if it's coming from market or from intermediate line
                match production_line
                    .import_src_intermediate_pipe(&item.type_id)
                {
                    // add item to every location along the pipe, including source
                    Some(pipe) => {
                        for location in pipe.locations() {
                            *assets_target
                                .entry(location.id())
                                .or_insert_with(HashMap::new)
                                .entry(item)
                                .or_insert(0) += quantity * max_num_builds;
                        }
                    }
                    // add item to every unique location along the pipe, except for the markets themselves
                    None => {
                        if !item.is_marketable() {
                            panic!("material not marketable");
                        }
                        for location in market_locations.iter() {
                            *assets_target
                                .entry(location.id())
                                .or_insert_with(HashMap::new)
                                .entry(item)
                                .or_insert(0) += quantity * max_num_builds;
                        }
                    }
                }
            }
        }
    }
    assets_target
}

pub fn build_in_locations<'cfg, 'db, 'api>(
    locations: &[Rc<Location<'cfg, 'db, 'api>>],
    slots: &mut IndustrySlots,
    type_volumes: &HashMap<u32, f64>,
) {
    loop {
        let mut best = None;
        for location in locations.iter() {
            for production_line in
                location.production_lines().iter_export_product()
            {
                if production_line.can_build(slots) {
                    if let Some(profit) =
                        production_line.profit(None, None, type_volumes)
                    {
                        if profit
                            > best.as_ref().map(|(_, p)| *p).unwrap_or(0.0)
                        {
                            best = Some((production_line.clone(), profit));
                        }
                    }
                }
            }
        }
        match best {
            Some((production_line, _)) => {
                production_line.build(slots, type_volumes);
            }
            None => {
                break;
            }
        }
    }
}

pub fn new_locations<'cfg, 'db, 'api>(
    cfg_locations: &'cfg [config::Location],
    db_lines: &'db HashMap<u32, industry_db::Line>,
    adjusted_prices: &'api HashMap<u32, f64>,
    cost_indices: &'api HashMap<u32, config::ManufacturingValue>,
    market_orders: &'api HashMap<u64, HashMap<u32, api_data::TypeMarketOrders>>,
    assets: &'api HashMap<u64, HashMap<Item, i64>>,
) -> Vec<Rc<Location<'cfg, 'db, 'api>>> {
    let locations = cfg_locations
        .iter()
        .map(|l| {
            (
                l.id,
                Rc::new(Location::new(
                    l,
                    market_orders.get(&l.id),
                    assets.get(&l.id),
                )),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut routes = HashMap::<u32, Rc<DeliveryRoute<'cfg, 'db, 'api>>>::new();

    for cfg_location in cfg_locations.iter() {
        let location = &locations[&cfg_location.id];
        for (dst_location_id, cfg_route) in cfg_location.routes.iter() {
            let dst_location = locations[dst_location_id].clone();
            let route = Rc::new(DeliveryRoute::new(
                cfg_route,
                location.clone(),
                dst_location,
            ));
            routes.insert(cfg_route.id, route.clone());
            location.routes.borrow_mut().push(route);
        }
    }

    let mut pipes = HashMap::<u32, Rc<DeliveryPipe<'cfg, 'db, 'api>>>::new();

    for cfg_location in cfg_locations.iter() {
        let location = &locations[&cfg_location.id];
        for (cfg_pipe_id, cfg_pipe) in cfg_location.pipes.iter() {
            let pipe = Rc::new(DeliveryPipe::new(
                cfg_pipe
                    .iter()
                    .map(|cfg_route_id| routes[cfg_route_id].clone())
                    .collect(),
            ));
            pipes.insert(*cfg_pipe_id, pipe.clone());
            if pipe.src().id() == location.id() {
                location.export_pipes.borrow_mut().push(pipe.clone());
            } else if pipe.dst().id() == location.id() {
                location.import_pipes.borrow_mut().push(pipe.clone());
            } else {
                panic!("pipe not connected to location");
            }
        }
        // for (cfg_pipe_id, cfg_pipe) in cfg_location.pipes.iter() {
        //     let pipe = &pipes[cfg_pipe_id];
        //     for route in pipe.routes.iter() {
        //         route.pipes.borrow_mut().push(pipe.clone());
        //     }
        // }
    }

    let mut production_lines =
        HashMap::<u32, Rc<ProductionLine<'cfg, 'db, 'api>>>::new();

    for cfg_location in cfg_locations.iter() {
        let location = &locations[&cfg_location.id];
        let cfg_location_production = match &cfg_location.production {
            Some(p) => p,
            None => continue,
        };
        for cfg_production_line in
            cfg_location_production.production_lines.iter()
        {
            let export_pipe =
                pipes[&cfg_production_line.export_pipe_id].clone();
            let import_src_market_pipes = cfg_production_line
                .import_src_market_pipe_ids
                .iter()
                .map(|id| pipes[id].clone())
                .collect();
            let production_line = Rc::new(ProductionLine::new(
                cfg_production_line,
                export_pipe,
                import_src_market_pipes,
                &db_lines[&cfg_production_line.id],
                adjusted_prices,
                cost_indices,
            ));
            production_lines
                .insert(cfg_production_line.id, production_line.clone());
            location
                .production
                .as_ref()
                .unwrap()
                .production_lines
                .borrow_mut()
                .entry(production_line.product())
                .or_insert_with(Vec::new)
                .push(production_line);
        }
        for cfg_production_line in
            cfg_location_production.production_lines.iter()
        {
            let production_line =
                production_lines[&cfg_production_line.id].clone();
            for (&type_id, cfg_import_production_line_id) in
                cfg_production_line.import_src_production_line_ids.iter()
            {
                let import_production_line =
                    production_lines[cfg_import_production_line_id].clone();
                production_line
                    .import_src_intermediate_production_lines
                    .borrow_mut()
                    .insert(type_id, import_production_line);
            }
        }
    }

    let locations = locations.into_values().collect::<Vec<_>>();

    let mut assets_target = assets_target(&locations);

    for location in locations.iter() {
        *location.assets_target.borrow_mut() = assets_target
            .remove(&location.id())
            .unwrap_or(HashMap::new());
    }

    locations
}

pub struct Location<'cfg, 'db, 'api> {
    pub inner: &'cfg config::Location,
    pub routes: RefCell<Vec<Rc<DeliveryRoute<'cfg, 'db, 'api>>>>,
    pub import_pipes: RefCell<Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>>,
    pub export_pipes: RefCell<Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>>,
    pub production: Option<LocationProduction<'cfg, 'db, 'api>>,
    pub market: Option<LocationMarket<'cfg, 'api>>,
    pub assets: Option<&'api HashMap<Item, i64>>,
    pub assets_target: RefCell<HashMap<Item, i64>>,
    pub assets_consumed: RefCell<HashMap<Item, i64>>,
}

impl<'cfg, 'db, 'api> Location<'cfg, 'db, 'api> {
    pub fn new(
        inner: &'cfg config::Location,
        orders: Option<&'api HashMap<u32, api_data::TypeMarketOrders>>,
        assets: Option<&'api HashMap<Item, i64>>,
    ) -> Self {
        Self {
            inner,
            routes: RefCell::new(Vec::new()),
            import_pipes: RefCell::new(Vec::new()),
            export_pipes: RefCell::new(Vec::new()),
            production: inner
                .production
                .as_ref()
                .map(|p| LocationProduction::new(p)),
            market: inner
                .market
                .as_ref()
                .map(|m| LocationMarket::new(m, orders)),
            assets,
            assets_target: RefCell::new(HashMap::new()),
            assets_consumed: RefCell::new(HashMap::new()),
        }
    }

    pub fn id(&self) -> u64 {
        self.inner.id
    }

    pub fn system_id(&self) -> u32 {
        self.inner.system_id
    }

    pub fn unwrap_market(&self) -> &LocationMarket<'cfg, 'api> {
        self.market.as_ref().unwrap()
    }

    pub fn unwrap_production(&self) -> &LocationProduction<'cfg, 'db, 'api> {
        self.production.as_ref().unwrap()
    }

    pub fn num_target(&self, item: Item) -> i64 {
        self.assets_target.borrow().get(&item).copied().unwrap_or(0)
    }

    pub fn consume_for_build(&self, item: Item, quantity: i64) {
        *self.assets_consumed.borrow_mut().entry(item).or_insert(0) += quantity;
    }

    pub fn num_consumed_for_build(&self, item: Item) -> i64 {
        self.assets_consumed
            .borrow()
            .get(&item)
            .copied()
            .unwrap_or(0)
    }

    pub fn production_lines(
        &self,
    ) -> LocationProductionLines<'_, 'cfg, 'db, 'api> {
        match self.production.as_ref() {
            Some(p) => p.production_lines(),
            None => LocationProductionLines { inner: None },
        }
    }

    pub fn num_available(&self, context: Option<u64>, item: Item) -> i64 {
        // add number of item present in assets
        let mut available = self
            .assets
            .map(|a| a.get(&item).copied())
            .flatten()
            .unwrap_or(0);

        // add number of item purchased from market
        if item.is_marketable() {
            available += self
                .market
                .as_ref()
                .map(|m| m.orders.num_purchased(context, &item.type_id))
                .unwrap_or(0);
        }

        // add number of item built
        available += self
            .production
            .as_ref()
            .map(|p| {
                p.production_lines.borrow().get(&item).map(|pls| {
                    pls.iter().map(|pl| pl.num_building()).sum::<i64>()
                })
            })
            .flatten()
            .unwrap_or(0);

        // add number of item imported via delivery pipes
        available += self
            .import_pipes
            .borrow()
            .iter()
            .map(|pipe| {
                pipe.deliveries.borrow().get(&item).copied().unwrap_or(0)
            })
            .sum::<i64>();

        // subtract number of item exported via delivery pipes
        available -= self
            .export_pipes
            .borrow()
            .iter()
            .map(|pipe| {
                pipe.deliveries.borrow().get(&item).copied().unwrap_or(0)
            })
            .sum::<i64>();

        // subtract number of item consumed for building
        available -= self.num_consumed_for_build(item);

        available
    }
}

pub struct LocationProduction<'cfg, 'db, 'api> {
    pub inner: &'cfg config::LocationProduction,
    pub production_lines:
        RefCell<HashMap<Item, Vec<Rc<ProductionLine<'cfg, 'db, 'api>>>>>,
}

impl<'cfg, 'db, 'api> LocationProduction<'cfg, 'db, 'api> {
    pub fn new(inner: &'cfg config::LocationProduction) -> Self {
        Self {
            inner,
            production_lines: RefCell::new(HashMap::new()),
        }
    }

    pub fn production_lines(
        &self,
    ) -> LocationProductionLines<'_, 'cfg, 'db, 'api> {
        LocationProductionLines {
            inner: Some(self.production_lines.borrow()),
        }
    }
}

pub struct LocationProductionLines<'lp, 'cfg, 'db, 'api> {
    inner: Option<
        Ref<'lp, HashMap<Item, Vec<Rc<ProductionLine<'cfg, 'db, 'api>>>>>,
    >,
}

impl<'lp, 'cfg, 'db, 'api> LocationProductionLines<'lp, 'cfg, 'db, 'api> {
    pub fn iter_all(
        &self,
    ) -> impl Iterator<Item = &Rc<ProductionLine<'cfg, 'db, 'api>>> {
        self.inner
            .as_ref()
            .map(|pls_map| pls_map.values().map(|pls| pls.iter()))
            .into_iter()
            .flatten()
            .flatten()
    }

    pub fn iter_export_product(
        &self,
    ) -> impl Iterator<Item = &Rc<ProductionLine<'cfg, 'db, 'api>>> {
        self.iter_all()
            .filter(|pl| pl.export_kind() == ProductionLineExportKind::Product)
    }

    pub fn iter_export_intermediate(
        &self,
    ) -> impl Iterator<Item = &Rc<ProductionLine<'cfg, 'db, 'api>>> {
        self.iter_all().filter(|pl| {
            pl.export_kind() == ProductionLineExportKind::Intermediate
        })
    }
}

pub struct LocationMarket<'cfg, 'api> {
    pub inner: &'cfg config::LocationMarket,
    pub orders: LocationMarketOrders<'api>,
}

impl<'cfg, 'api> LocationMarket<'cfg, 'api> {
    pub fn new(
        inner: &'cfg config::LocationMarket,
        orders: Option<&'api HashMap<u32, api_data::TypeMarketOrders>>,
    ) -> Self {
        Self {
            inner,
            orders: LocationMarketOrders::new(orders),
        }
    }
}
