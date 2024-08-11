use super::*;

// decryptor can be derived from existing information
#[derive(Debug, Clone, Deserialize)]
pub struct ProductionLine {
    pub id: u32,
    #[serde(flatten)]
    pub transput: Transput,
    pub kind: ManufacturingKind,
    pub export_kind: ProductionLineExportKind,
    pub export_pipe_id: u32,
    pub import_src_market_pipe_ids: Vec<u32>,
    pub import_src_production_line_ids: HashMap<u32, u32>,
    pub decryptor: Option<u32>,
    pub parallel: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ProductionLineExportKind {
    Product,
    Intermediate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ManufacturingKind {
    Manufacturing,
    Invention,
    Reaction,
    Copy,
}

impl ManufacturingKind {
    pub fn from_activity_id(id: i32) -> Option<ManufacturingKind> {
        unimplemented!()
    }

    pub const fn is_science(&self) -> bool {
        match self {
            ManufacturingKind::Invention => true,
            ManufacturingKind::Copy => true,
            ManufacturingKind::Manufacturing => false,
            ManufacturingKind::Reaction => false,
        }
    }
}

impl Into<IndustrySlot> for ManufacturingKind {
    fn into(self) -> IndustrySlot {
        match self {
            ManufacturingKind::Copy => IndustrySlot::Science,
            ManufacturingKind::Invention => IndustrySlot::Science,
            ManufacturingKind::Reaction => IndustrySlot::Reaction,
            ManufacturingKind::Manufacturing => IndustrySlot::Manufacturing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct Transput {
    pub blueprint: Item,
    pub product: Item,
}

impl Transput {
    pub fn new(blueprint: Item, product: Item) -> Transput {
        Transput {
            blueprint: blueprint,
            product: product,
        }
    }

    pub const fn null() -> Transput {
        Transput {
            blueprint: Item::null(),
            product: Item::null(),
        }
    }
}
