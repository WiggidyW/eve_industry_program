use serde::Deserialize;

use super::*;

// the items could just be u32 in this case? contracts would need item though
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub id: u64,
    pub name: String,
    pub system_id: u32,
    pub production: Option<LocationProduction>,
    pub market: Option<LocationMarket>,
    pub routes: HashMap<u64, DeliveryRoute>,
    pub pipes: HashMap<u32, Vec<u32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationProduction {
    pub tax: ManufacturingValue,
    pub rigs: [Option<u32>; 3],
    pub structure_type_id: u32,
    pub production_lines: Vec<ProductionLine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationMarket {
    pub sales_tax: f64,
    pub brokers_fee: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ManufacturingValue {
    manufacturing: f64,
    invention: f64,
    reaction: f64,
    copy: f64,
}

impl ManufacturingValue {
    pub fn kind_value(&self, kind: ManufacturingKind) -> f64 {
        match kind {
            ManufacturingKind::Manufacturing => self.manufacturing,
            ManufacturingKind::Invention => self.invention,
            ManufacturingKind::Reaction => self.reaction,
            ManufacturingKind::Copy => self.copy,
        }
    }
}
