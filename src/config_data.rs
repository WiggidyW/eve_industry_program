use crate::{common::*, static_data::*};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

pub struct ConfigDataContext {
    pub locations: Locations,
    pub delivery_pipes: DeliveryPipes,
    pub production_lines: ProductionLines,
    pub max_time: Duration, // time that all production lines are running for
    pub daily_flex_time: Duration, // extra time required for daily startables under 24 hours
}

pub enum Rigs {
    Zero,
    One(TypeId),
    Two(TypeId, TypeId),
    Three(TypeId, TypeId, TypeId),
}

pub struct Location {
    pub rigs: Rigs,
    pub production_lines: Vec<usize>,
}

pub struct Locations(Vec<Location>);

pub struct DeliveryPipeRoute {
    pub src_location: usize,
    pub dst_location: usize,
    pub m3_rate: f64,         // m3_fee = m3_rate * m3
    pub collateral_rate: f64, // collateral_fee = collateral_rate * collateral
}

pub struct DeliveryPipe(Vec<DeliveryPipeRoute>);

pub struct DeliveryPipes(Vec<DeliveryPipe>);

pub enum ProductionLineExportKind {
    Product,
    IntermediateMaterial,
}

// pub struct ProductionLineExport {
//     pub type_id: TypeId,
//     pub quantity: u32,
// }

pub enum ProductionLineBlueprint {
    BPO {
        me: MaterialEfficiency,
        te: TimeEfficiency,
    },
    BPC {
        production_line: usize,
    },
}

pub struct ProductionLine {
    pub type_id: TypeId,
    pub build_kind: BuildKind,
    pub export_kind: ProductionLineExportKind,
    pub export_dst_location: usize, // index into Locations
    pub import_src_market_locations: Vec<usize>, // index into Locations
    pub import_src_intermediate_production_lines: HashMap<TypeId, usize>, // index into ProductionLines
    pub blueprint: ProductionLineBlueprint,
    pub location: usize,
    pub decryptor: Option<Decryptor>,
}

pub struct ProductionLines(Vec<ProductionLine>);

impl ProductionLine {
    pub fn max_slot_usage(&self) -> SlotCount {
        unimplemented!()
    }

    pub fn time_per_run(&self) -> Duration {
        unimplemented!()
    }

    // (runs per sequence, number of sequences)
    pub fn num_runs(&self, ctx: &ConfigDataContext) -> (u32, u32) {
        let time_per_run: Duration = self.time_per_run();
        match self.blueprint {
            ProductionLineBlueprint::BPO { me: _, te: _ } => {
                let num_runs = ctx.max_time.as_secs() / time_per_run.as_secs();
                if num_runs > u32::MAX as u64 {
                    panic!("num_runs > u32::MAX");
                }
                (num_runs as u32, 1)
            }
            ProductionLineBlueprint::BPC { production_line: _ } => {
                unimplemented!()
            }
        }
    }

    pub fn total_num_runs(&self, ctx: &ConfigDataContext) -> u32 {
        let (runs_per_sequence, num_sequences) = self.num_runs(ctx);
        runs_per_sequence * num_sequences
    }

    pub fn 
}

// Utility Data
