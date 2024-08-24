use super::*;
use crate::config::{self, IndustrySlots, Item, ProductionLineExportKind};
use crate::industry_db;
use core::f64;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct ProductionLine<'cfg, 'db, 'api> {
    pub inner: &'cfg config::ProductionLine,
    pub export_pipe: Rc<DeliveryPipe<'cfg, 'db, 'api>>,
    pub import_src_market_pipes: Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>,
    pub import_src_intermediate_production_lines:
        RefCell<HashMap<u32, Rc<ProductionLine<'cfg, 'db, 'api>>>>,
    pub db_line: DbLineTransformed<'db>,
    pub installation_cost: f64, // installation cost for N runs
    pub builds: RefCell<i64>,
}

impl<'cfg, 'db, 'api> ProductionLine<'cfg, 'db, 'api> {
    pub fn new(
        inner: &'cfg config::ProductionLine,
        export_pipe: Rc<DeliveryPipe<'cfg, 'db, 'api>>,
        import_src_market_pipes: Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>,
        db_line: &'db industry_db::Line,
        adjusted_prices: &'api HashMap<u32, f64>,
        cost_indices: &'api HashMap<u32, config::ManufacturingValue>,
        max_time: Duration,
        daily_flex_time: Duration,
    ) -> Self {
        let db_line =
            DbLineTransformed::new(db_line, max_time, daily_flex_time);
        let installation_cost = {
            let eiv = db_line
                .installation_minerals()
                .map(|(item, quantity)| {
                    let price = adjusted_prices.get(&item.type_id).unwrap();
                    *price * quantity as f64
                })
                .sum::<f64>();
            let index_cost = cost_indices[&export_pipe.src().system_id()]
                .kind_value(inner.kind);
            eiv * index_cost * db_line.cost_multiplier()
        };
        Self {
            inner,
            installation_cost,
            export_pipe,
            import_src_market_pipes,
            import_src_intermediate_production_lines: RefCell::new(
                HashMap::new(),
            ),
            db_line,
            builds: RefCell::new(0),
        }
    }

    pub fn runs(&self) -> i64 {
        self.db_line.runs()
    }

    pub fn portion(&self) -> i64 {
        self.db_line.portion()
    }

    pub fn runs_per(&self) -> f64 {
        self.runs() as f64 / self.portion() as f64
    }

    pub fn runs_for(&self, num_produced: f64) -> f64 {
        self.runs_per() * num_produced
    }

    pub fn decryptor(&self) -> Option<Item> {
        self.inner.decryptor.map(|type_id| Item::new(type_id))
    }

    pub fn installation_cost_for(&self, num_produced: f64) -> f64 {
        self.installation_cost * (num_produced / self.portion() as f64)
    }

    pub fn import_src_intermediate_production_line(
        &self,
        type_id: &u32,
    ) -> Option<Rc<ProductionLine<'cfg, 'db, 'api>>> {
        self.import_src_intermediate_production_lines
            .borrow()
            .get(type_id)
            .map(|pl| pl.clone())
    }

    pub fn import_src_intermediate_pipe(
        &self,
        type_id: &u32,
    ) -> Option<Rc<DeliveryPipe<'cfg, 'db, 'api>>> {
        self.import_src_intermediate_production_lines
            .borrow()
            .get(type_id)
            .map(|pl| pl.export_pipe.clone())
    }

    pub fn export_pipe(&self) -> &DeliveryPipe<'cfg, 'db, 'api> {
        self.export_pipe.as_ref()
    }

    pub fn location(&self) -> &Location<'cfg, 'db, 'api> {
        self.export_pipe.src()
    }

    pub fn location_production(&self) -> &LocationProduction<'cfg, 'db, 'api> {
        self.location().production.as_ref().unwrap()
    }

    fn minerals(
        &self,
        num_produced: Option<f64>,
    ) -> impl Iterator<Item = (config::Item, f64)> + '_ {
        let mult = match num_produced {
            Some(num_produced) => num_produced / self.portion() as f64,
            None => 1.0,
        };
        self.db_line
            .minerals()
            .map(move |(item, quantity)| (item, quantity as f64 * mult))
    }

    pub fn minerals_i64(
        &self,
    ) -> impl Iterator<Item = (config::Item, i64)> + '_ {
        self.db_line.minerals()
    }

    fn import_src_market_locations<'this>(
        &'this self,
    ) -> impl Iterator<Item = &'this LocationMarket<'cfg, 'api>> + 'db
    where
        'this: 'db,
    {
        self.import_src_market_pipes
            .iter()
            .map(move |pipe| pipe.src().unwrap_market())
    }

    pub fn import_src_market_pipes(
        &self,
    ) -> impl Iterator<Item = &DeliveryPipe<'cfg, 'db, 'api>> {
        self.import_src_market_pipes
            .iter()
            .map(|pipe| pipe.as_ref())
    }

    fn import_src_market_pipes_with_orders(
        &self,
    ) -> impl Iterator<
        Item = (&DeliveryPipe<'cfg, 'db, 'api>, &LocationMarketOrders<'api>),
    > {
        self.import_src_market_pipes.iter().map(|pipe| {
            let orders = &pipe.src().unwrap_market().orders;
            (pipe.as_ref(), orders)
        })
    }

    fn import_src_market_rates_with_orders<'this>(
        &'this self,
    ) -> impl Iterator<Item = (config::DeliveryRate, &LocationMarketOrders<'api>)>
           + 'this
    where
        'this: 'db,
        'this: 'cfg,
    {
        self.import_src_market_pipes.iter().map(|pipe| {
            let rate = pipe.delivery_rate();
            let orders = &pipe.src().unwrap_market().orders;
            (rate, orders)
        })
    }

    pub fn product(&self) -> Item {
        self.inner.transput.product
    }

    pub fn blueprint(&self) -> Item {
        self.inner.transput.blueprint
    }

    pub fn num_builds(&self) -> i64 {
        *self.builds.borrow()
    }

    pub fn max_num_builds(&self) -> i64 {
        self.inner.parallel
    }

    pub fn num_buildable(&self) -> i64 {
        self.portion() * self.max_num_builds()
    }

    pub fn num_building(&self) -> i64 {
        self.portion() * self.num_builds()
    }

    fn unwrap_export_market(&self) -> &LocationMarket<'cfg, 'api> {
        self.export_pipe.dst().unwrap_market()
    }

    fn export_location(&self) -> &Location<'cfg, 'db, 'api> {
        self.export_pipe.dst()
    }

    pub fn job_kind(&self) -> config::ManufacturingKind {
        self.inner.kind
    }

    pub fn slot_kind(&self) -> config::IndustrySlot {
        self.job_kind().into()
    }

    pub fn export_kind(&self) -> config::ProductionLineExportKind {
        self.inner.export_kind
    }

    pub fn max_slots(&self) -> config::IndustrySlots {
        let mut slots = IndustrySlots::from_slot(self.slot_kind());
        for sub_production_line in self
            .import_src_intermediate_production_lines
            .borrow()
            .values()
        {
            slots.add(sub_production_line.max_slots());
        }
        slots
    }

    fn permanent_reserve_from_market_and_deliver(
        &self,
        item: &Item,
        quantity: i64,
        type_volumes: &HashMap<Item, f64>,
    ) {
        let volume = type_volumes.get(&item).copied().unwrap_or(0.0);
        let mut reserved = 0;
        while reserved < quantity {
            let mut cheapest_market = None;
            let mut cheapest_reservable = 0.0;
            let mut cheapest_price_with_delivery = f64::INFINITY;
            for (pipe, orders) in self.import_src_market_pipes_with_orders() {
                if let Some(order) = orders.next_available(None, &item.type_id)
                {
                    let price_with_delivery = order.price
                        + pipe.delivery_rate().m3_rate * volume
                        + pipe.delivery_rate().collateral_rate * order.price;
                    if price_with_delivery < cheapest_price_with_delivery {
                        cheapest_market = Some((orders, pipe));
                        cheapest_reservable = order.volume;
                        cheapest_price_with_delivery = price_with_delivery;
                    }
                }
            }
            if let Some((orders, pipe)) = cheapest_market {
                let cheapest_reserve =
                    (cheapest_reservable as i64).min(quantity - reserved);
                reserved += cheapest_reserve;
                orders.reserve_i64(None, &item.type_id, cheapest_reserve);
                pipe.deliver(*item, cheapest_reserve);
            } else {
                let mut highest_volume_market = None;
                let mut highest_volume = 0.0;
                for (pipe, orders) in self.import_src_market_pipes_with_orders()
                {
                    let volume = orders.total_volume(&item.type_id);
                    if volume > highest_volume
                        || highest_volume_market.is_none()
                    {
                        highest_volume_market = Some((orders, pipe));
                        highest_volume = volume;
                    }
                }
                if let Some((orders, pipe)) = highest_volume_market {
                    orders.reserve_i64(
                        None,
                        &item.type_id,
                        quantity - reserved,
                    );
                    pipe.deliver(*item, quantity - reserved);
                    reserved = quantity;
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn reserve_from_market(
        &self,
        context: u64,
        item: &Item,
        quantity: f64,
        type_volumes: &HashMap<Item, f64>,
    ) -> Option<f64> {
        let volume = type_volumes.get(&item).copied().unwrap_or(0.0);
        let mut reserved = 0.0;
        let mut type_cost = 0.0;
        while reserved < quantity {
            let mut cheapest_market: Option<&LocationMarketOrders<'api>> = None;
            let mut cheapest_reservable = 0.0;
            let mut cheapest_price_with_delivery = f64::INFINITY;
            for (pipe, orders) in self.import_src_market_pipes_with_orders() {
                if let Some(order) =
                    orders.next_available(Some(context), &item.type_id)
                {
                    let price_with_delivery = order.price
                        + pipe.delivery_rate().m3_rate * volume
                        + pipe.delivery_rate().collateral_rate * order.price;
                    if price_with_delivery < cheapest_price_with_delivery {
                        cheapest_market = Some(orders);
                        cheapest_reservable = order.volume;
                        cheapest_price_with_delivery = price_with_delivery;
                    }
                }
            }
            let orders = cheapest_market?;
            let cheapest_reserve = cheapest_reservable.min(quantity - reserved);
            reserved += cheapest_reserve;
            type_cost += cheapest_price_with_delivery * cheapest_reserve;
            orders.reserve(Some(context), &item.type_id, cheapest_reserve);
        }
        Some(type_cost)
    }

    fn market_cost_with_delivery(
        &self,
        context: u64,
        num_produced: Option<f64>,
        type_volumes: &HashMap<Item, f64>,
    ) -> Option<f64> {
        let mut cost = 0.0;
        for (item, quantity) in self.minerals(num_produced) {
            match self.import_src_intermediate_production_line(&item.type_id) {
                Some(_) => continue,
                None => match self.reserve_from_market(
                    context,
                    &item,
                    quantity,
                    type_volumes,
                ) {
                    Some(type_cost) => cost += type_cost,
                    None => return None,
                },
            }
        }
        Some(cost)
    }

    fn profit_context(&self) -> u64 {
        ((self.inner.id as u64) << 32) | (*self.builds.borrow() as u64)
    }

    fn revenue_with_delivery(
        &self,
        num_produced: Option<f64>,
        type_volumes: &HashMap<Item, f64>,
        market_cost_with_delivery: f64,
    ) -> Profit {
        let num_produced =
            num_produced.unwrap_or(self.db_line.portion() as f64);
        let volume = type_volumes.get(&self.product()).copied().unwrap_or(0.0);
        let delivery_rate = self.export_pipe().delivery_rate();
        let min_sell = match self.export_kind() {
            config::ProductionLineExportKind::Product => self
                .unwrap_export_market()
                .orders
                .min_sell(&self.product().type_id),
            config::ProductionLineExportKind::Intermediate => None,
        };
        let delivery_m3_fee = delivery_rate.m3_rate * volume * num_produced;
        let delivery_collateral_fee = delivery_rate.collateral_rate
            * match min_sell {
                Some(min_sell) => min_sell * num_produced,
                None => {
                    market_cost_with_delivery
                        + self.installation_cost_for(num_produced)
                }
            };
        let delivery_fee = delivery_m3_fee + delivery_collateral_fee;
        let market_revenue = min_sell.unwrap_or(0.0) * num_produced;
        let (sales_tax, brokers_fee) = match self.export_kind() {
            config::ProductionLineExportKind::Product => (
                self.unwrap_export_market().sales_tax() * market_revenue,
                self.unwrap_export_market().brokers_fee() * market_revenue,
            ),
            config::ProductionLineExportKind::Intermediate => (0.0, 0.0),
        };
        Profit::new(delivery_fee + sales_tax + brokers_fee, market_revenue)
    }

    pub fn profit(
        &self,
        context: Option<u64>,
        num_produced: Option<f64>,
        type_volumes: &HashMap<Item, f64>,
    ) -> Option<Profit> {
        let context = context.unwrap_or(self.profit_context());

        let market_cost_with_delivery = self.market_cost_with_delivery(
            context,
            num_produced,
            type_volumes,
        )?;

        let revenue_with_delivery = self.revenue_with_delivery(
            num_produced,
            type_volumes,
            market_cost_with_delivery,
        );

        let mut profit = revenue_with_delivery;
        profit.cost += market_cost_with_delivery;
        profit.cost += match num_produced {
            Some(num_produced) => self.installation_cost_for(num_produced),
            None => self.installation_cost,
        };

        for (item, quantity) in self.minerals(num_produced) {
            if let Some(pl) =
                self.import_src_intermediate_production_line(&item.type_id)
            {
                profit +=
                    pl.profit(Some(context), Some(quantity), type_volumes)?;
            }
        }

        Some(profit)
    }

    pub fn can_build(&self, slots: &IndustrySlots) -> bool {
        slots.can_use_slots(&self.max_slots())
            && self.num_builds() < self.max_num_builds()
            && self.should_build_and_deliver()
    }

    pub fn should_build_and_deliver(&self) -> bool {
        self.export_location().num_available(None, self.product())
            < self.export_location().num_target(self.product())
    }

    pub fn build(
        &self,
        slots: &mut IndustrySlots,
        type_volumes: &HashMap<Item, f64>,
    ) {
        // use build slot
        slots.use_slot(self.slot_kind());

        // increment builds
        *self.builds.borrow_mut() += 1;

        // purchase or buy and deliver minerals
        for (item, quantity) in self.minerals_i64() {
            match self.import_src_intermediate_production_line(&item.type_id) {
                Some(pl) => {
                    // generally, more will be built than delivered
                    // so, intermediate lines don't always run
                    if pl.should_build_and_deliver() {
                        pl.build(slots, type_volumes);
                        pl.export_pipe().deliver(item, quantity);
                    } else {
                        // import is not needed, we have enough already here
                    }
                }
                None => self.permanent_reserve_from_market_and_deliver(
                    &item,
                    quantity,
                    type_volumes,
                ),
            }
        }

        // export product if this is a product line
        if self.export_kind() == ProductionLineExportKind::Product {
            self.export_pipe().deliver(self.product(), self.portion());
        }
    }
}
