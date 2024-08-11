use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryRoute {
    pub id: u32,
    pub service_name: String,
    #[serde(flatten)]
    pub rate: DeliveryRate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryRate {
    pub m3_rate: f64,         // m3_fee = m3_rate * m3
    pub collateral_rate: f64, // collateral_fee = collateral_rate * collateral
}

impl DeliveryRate {
    pub fn new() -> Self {
        Self {
            m3_rate: 0.0,
            collateral_rate: 0.0,
        }
    }
}
