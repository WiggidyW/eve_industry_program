use super::*;
use crate::config::Item;
use serde::Serialize;

#[derive(Serialize)]
pub struct OutputLocations<'cfg, 'db> {
    // DOES NOT include delivery collateral rate costs
    cost: f64,
    locations: Vec<OutputLocation<'cfg, 'db>>,
}

impl<'cfg, 'db> OutputLocations<'cfg, 'db> {
    pub fn new(
        locations: &[Rc<Location<'cfg, '_, '_>>],
        type_names: &'db HashMap<Item, String>,
        type_volumes: &'db HashMap<Item, f64>,
    ) -> Self {
        let mut cost = 0.0;
        let locations = locations
            .iter()
            .map(|location| {
                OutputLocation::new(
                    location,
                    type_names,
                    type_volumes,
                    &mut cost,
                )
            })
            .collect();
        Self { cost, locations }
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::create("output.json")?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct OutputLocation<'cfg, 'db> {
    pub location: &'cfg str,
    pub purchases: Option<Vec<OutputPurchase<'db>>>,
    pub builds: Option<Vec<OutputBuild<'db>>>,
    pub deliveries: Option<Vec<OutputDeliveries<'cfg, 'db>>>,
    pub missing_assets: Option<Vec<OutputAssetTarget<'db>>>,
}

impl<'cfg, 'db> OutputLocation<'cfg, 'db> {
    pub fn new(
        location: &Location<'cfg, '_, '_>,
        type_names: &'db HashMap<Item, String>,
        type_volumes: &'db HashMap<Item, f64>,
        cost: &mut f64,
    ) -> Self {
        Self {
            location: location.name(),
            purchases: Self::purchases(location, type_names, cost),
            builds: Self::builds(location, type_names, cost),
            deliveries: Self::deliveries(
                location,
                type_names,
                type_volumes,
                cost,
            ),
            missing_assets: Self::missing_assets(location, type_names),
        }
    }

    fn purchases(
        location: &Location<'cfg, '_, '_>,
        type_names: &'db HashMap<Item, String>,
        cost: &mut f64,
    ) -> Option<Vec<OutputPurchase<'db>>> {
        let location_market = match &location.market {
            Some(location_market) => location_market,
            None => return None,
        };
        let mut purchases = Vec::new();
        for (type_id, quantity, stats) in
            location_market.orders.iter_purchases(None, cost)
        {
            purchases.push(OutputPurchase {
                item: &type_names[&Item::new(type_id)],
                quantity,
                price_low: stats.price_low,
                price_high: stats.price_high,
            });
        }
        Some(purchases)
    }

    fn builds(
        location: &Location<'cfg, '_, '_>,
        type_names: &'db HashMap<Item, String>,
        cost: &mut f64,
    ) -> Option<Vec<OutputBuild<'db>>> {
        let mut builds = None;
        for production_line in location.production_lines().iter_all() {
            let num_builds = production_line.num_builds();
            if num_builds == 0 {
                continue;
            }
            *cost += num_builds as f64 * production_line.installation_cost();
            builds.get_or_insert_with(Vec::new).push(OutputBuild {
                product: &type_names[&production_line.product()],
                blueprint: &type_names[&production_line.blueprint()],
                decryptor: production_line
                    .decryptor()
                    .map(|item| type_names[&item].as_str()),
                runs: production_line.runs_per_sequence(),
                sequences: production_line.num_sequences(),
                builds: num_builds,
            });
        }
        builds
    }

    fn deliveries(
        location: &Location<'cfg, '_, '_>,
        type_names: &'db HashMap<Item, String>,
        type_volumes: &'db HashMap<Item, f64>,
        cost: &mut f64,
    ) -> Option<Vec<OutputDeliveries<'cfg, 'db>>> {
        let mut deliveries_map = HashMap::new();
        for (route, delivery_pipes) in location
            .routes()
            .iter_transit(location.id())
            .map(|route| (route, route.pipes()))
        {
            for delivery_pipe in delivery_pipes.iter() {
                for (item, quantity) in delivery_pipe.deliveries().iter() {
                    let item_volume = *type_volumes.get(&item).unwrap_or(&0.0);
                    *cost += delivery_pipe.delivery_rate().m3_rate
                        * item_volume
                        * quantity as f64;
                    *deliveries_map
                        .entry((route.dst.name(), route.service_name()))
                        .or_insert(HashMap::new())
                        .entry(item)
                        .or_insert(0) += quantity;
                }
            }
        }

        let mut deliveries = None;
        for ((destination, service_name), item_deliveries) in deliveries_map {
            deliveries
                .get_or_insert_with(Vec::new)
                .push(OutputDeliveries {
                    destination,
                    service_name,
                    items: item_deliveries
                        .into_iter()
                        .map(|(item, quantity)| OutputDelivery {
                            item: &type_names[&item],
                            quantity,
                        })
                        .collect(),
                });
        }

        deliveries
    }

    fn missing_assets(
        location: &Location<'cfg, '_, '_>,
        type_names: &'db HashMap<Item, String>,
    ) -> Option<Vec<OutputAssetTarget<'db>>> {
        let mut missing_assets = None;
        for (item, target) in location.assets_target().iter() {
            let current = location.asset_quantity(item);
            if current < target {
                missing_assets.get_or_insert_with(Vec::new).push(
                    OutputAssetTarget {
                        item: &type_names[&item],
                        target,
                        current,
                    },
                );
            }
        }
        missing_assets
    }
}

#[derive(Serialize)]
pub struct OutputPurchase<'db> {
    pub item: &'db str,
    pub quantity: i64,
    pub price_low: f64,
    pub price_high: f64,
}

#[derive(Serialize)]
pub struct OutputBuild<'db> {
    pub product: &'db str,
    pub blueprint: &'db str,
    pub decryptor: Option<&'db str>,
    pub runs: i64,
    pub sequences: i64,
    pub builds: i64,
}

#[derive(Serialize)]
pub struct OutputDeliveries<'cfg, 'db> {
    pub destination: &'cfg str,
    pub service_name: &'cfg str,
    pub items: Vec<OutputDelivery<'db>>,
}

#[derive(Serialize)]
pub struct OutputDelivery<'db> {
    pub item: &'db str,
    pub quantity: i64,
}

#[derive(Serialize)]
pub struct OutputAssetTarget<'db> {
    pub item: &'db str,
    pub target: i64,
    pub current: i64,
}
