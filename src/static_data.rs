// pub struct Blueprint {}

// pub struct Blueprints(Vec<Blueprint>);

// impl Blueprint {}

use crate::common::*;
use std::collections::HashMap;

pub struct Decryptor;

pub fn per_run_raw_job_materials(
    product: TypeId,
    job_kind: JobKind,
    decryptor: Option<TypeId>,
) -> HashMap<TypeId, u64> {
    unimplemented!()
}

pub fn type_volume(type_id: TypeId) -> f64 {
    unimplemented!()
}

pub fn per_run_product_quantity(
    product: TypeId,
    job_kind: JobKind,
    decryptor: Option<TypeId>,
) -> f64 {
    unimplemented!()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Skill {
    pub kind: TypeId,
    pub level: u8,
}

impl Skill {
    pub fn te(&self, item: Item, kind: JobKind) -> f64 {
        unimplemented!()
    }
    pub fn me(&self, item: Item, kind: JobKind) -> f64 {
        1.0
    }
    pub fn ce(&self, item: Item, kind: JobKind) -> f64 {
        1.0
    }
}
