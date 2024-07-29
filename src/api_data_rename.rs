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
