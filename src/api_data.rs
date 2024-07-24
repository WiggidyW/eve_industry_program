use crate::common::*;
use std::collections::HashMap;

pub struct MarketOrders(HashMap<TypeId, TypeMarketOrders>);

pub struct TypeMarketOrders {
    orders: Vec<MarketOrder>,
    reserved: u64,
    total: u64,
}

// gonna need another struct for temporary reserves ^ when calcing profit inclusive of intermediate prod lines

pub struct MarketOrder {
    pub price: f64,
    pub volume: u32,
}

pub struct SystemCostIndices {
    pub manufacturing: f64,
    pub copying: f64,
    pub invention: f64,
    // pub reverse_engineering: f64,
    pub reaction: f64,
}
