use super::*;

// the items could just be typeid in this case? contracts would need item though
pub struct Location {
    pub id: u64,
    pub system_id: u32,
    pub name: String,
    pub production: Option<LocationProduction>,
    pub market: Option<LocationMarket>,
    pub routes: HashMap<u64, DeliveryRoute>,
    pub pipes: HashMap<u32, Vec<u32>>,
}

pub struct LocationProduction {
    pub tax: ManufacturingValue,
    pub rigs: [Option<TypeId>; 3],
    pub structure_type_id: TypeId,
    pub production_lines: Vec<ProductionLine>,
}

pub struct LocationMarket {
    pub sales_tax: f64,
    pub broker_fee: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
