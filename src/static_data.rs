// pub struct Blueprint {}

// pub struct Blueprints(Vec<Blueprint>);

// impl Blueprint {}

use crate::common::*;
use std::{collections::HashMap, time::Duration};

pub fn per_run_raw_job_materials(
    blueprint: Blueprint,
    product: TypeId,
    kind: JobKind,
    decryptor: Option<TypeId>,
) -> HashMap<TypeId, u64> {
    unimplemented!()
}

pub fn type_volume(type_id: TypeId) -> f64 {
    unimplemented!()
}

pub fn per_run_product_quantity(
    blueprint: Blueprint,
    product: TypeId,
    kind: JobKind,
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
    pub fn te(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        unimplemented!()
    }
    pub fn me(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        1.0
    }
    pub fn ce(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        1.0
    }
}

pub struct Rig(TypeId);

impl Rig {
    pub fn te(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        unimplemented!()
    }
    pub fn me(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        unimplemented!()
    }
    pub fn ce(
        &self,
        blueprint: Blueprint,
        product: Item,
        kind: JobKind,
        decryptor: Option<TypeId>,
    ) -> f64 {
        unimplemented!()
    }
}

pub fn raw_time_per_run(
    blueprint: Blueprint,
    product: TypeId,
    kind: JobKind,
    decryptor: Option<TypeId>,
) -> Duration {
    unimplemented!()
}

pub fn per_run_num_produced(
    blueprint: Blueprint,
    product: TypeId,
    kind: JobKind,
    decryptor: Option<TypeId>,
) -> f64 {
    unimplemented!()
}

pub enum Security {
    High,
    Low,
    Null,
}

pub enum Structure {
    Structure(TypeId),
    Station,
}
