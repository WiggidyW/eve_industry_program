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
