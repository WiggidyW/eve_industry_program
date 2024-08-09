use crate::config::{self, Item};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MarketOrder {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TypeMarketOrders {
    pub orders: Vec<MarketOrder>, // cheapest first
    pub total: f64,
}

pub struct Api {
    pub adjusted_prices: HashMap<u32, f64>,
    pub cost_indices: HashMap<u32, config::ManufacturingValue>,
    pub market_orders: HashMap<u64, HashMap<u32, TypeMarketOrders>>,
    pub assets: HashMap<u64, HashMap<Item, i64>>,
}
