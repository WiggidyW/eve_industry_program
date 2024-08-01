use crate::{
    api_data_rename as api_data,
    config::{self, Item},
    industry_db,
};
use std::{collections::HashMap, rc::Rc};

mod delivery_route;

use delivery_route::*;

mod location;
use location::*;

mod delivery_pipe;
use delivery_pipe::*;

mod production_line;
use production_line::*;

mod market_orders;
use market_orders::*;

pub struct RuntimeData<'cfg, 'db, 'api> {
    pub locations: Vec<Rc<Location<'cfg, 'db, 'api>>>,
    pub type_volumes: &'db HashMap<u32, f64>,
}

impl<'cfg, 'db, 'api> RuntimeData<'cfg, 'db, 'api> {
    pub fn new(
        cfg_locations: &'cfg [config::Location],
        db_lines: &'db HashMap<u32, industry_db::Line>,
        type_volumes: &'db HashMap<u32, f64>,
        adjusted_prices: &'api HashMap<u32, f64>,
        cost_indices: &'api HashMap<u32, config::ManufacturingValue>,
        market_orders: &'api HashMap<
            u64,
            HashMap<u32, api_data::TypeMarketOrders>,
        >,
        assets: &'api HashMap<u64, HashMap<Item, i64>>,
    ) -> Self {
        Self {
            locations: new_locations(
                cfg_locations,
                db_lines,
                adjusted_prices,
                cost_indices,
                market_orders,
                assets,
            ),
            type_volumes,
        }
    }
}
