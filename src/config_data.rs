use crate::{common::*, static_data::*};
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
    time::Duration,
};

pub struct ConfigDataContext {
    pub locations: Locations,
    pub delivery_routes: DeliveryRoutes,
    pub delivery_pipes: DeliveryPipes,
    pub production_lines: ProductionLines,
    pub collateral_override: CollateralOverride,
    pub max_time: Duration, // time that all production lines are running for
    pub daily_flex_time: Duration, // extra time required for daily startables under 24 hours
}

pub enum Rigs {
    Zero,
    One(TypeId),
    Two(TypeId, TypeId),
    Three(TypeId, TypeId, TypeId),
}

pub struct CollateralOverride(HashMap<TypeId, f64>);

pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub system_id: SystemId,
    pub production: Option<LocationProduction>,
    pub market: Option<LocationMarket>,
}

impl Location {
    pub fn unwrap_market(&self) -> &LocationMarket {
        self.market.as_ref().unwrap()
    }
}

pub struct LocationProduction {
    pub rigs: Rigs,
    pub production_lines: Vec<ProductionLineId>,
}

pub struct LocationMarket {
    pub sales_tax: f64,
    pub broker_fee: f64,
}

pub struct Locations(HashMap<LocationId, Location>);

impl Index<LocationId> for Locations {
    type Output = Location;
    fn index(&self, index: LocationId) -> &Self::Output {
        &self.0[&index]
    }
}

pub struct DeliveryRate {
    pub m3_rate: f64,         // m3_fee = m3_rate * m3
    pub collateral_rate: f64, // collateral_fee = collateral_rate * collateral
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeliveryRouteId(u32);

pub struct DeliveryRoute {
    pub service_name: String,
    pub src_location: LocationId,
    pub dst_location: LocationId,
    pub rate: DeliveryRate,
}

pub struct DeliveryRoutes(HashMap<DeliveryRouteId, DeliveryRoute>);

impl Index<DeliveryRouteId> for DeliveryRoutes {
    type Output = DeliveryRoute;
    fn index(&self, index: DeliveryRouteId) -> &Self::Output {
        &self.0[&index]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeliveryPipeId(u32);

pub struct DeliveryPipe(Vec<DeliveryRouteId>);

impl DeliveryPipe {
    pub fn rate(&self, cfg_ctx: &ConfigDataContext) -> DeliveryRate {
        let mut m3_rate = 0.0;
        let mut collateral_rate = 0.0;
        for route in self.0.iter().map(|&route| &cfg_ctx.delivery_routes[route])
        {
            m3_rate += route.rate.m3_rate;
            collateral_rate += route.rate.collateral_rate;
        }
        DeliveryRate {
            m3_rate,
            collateral_rate,
        }
    }

    pub fn src_location<'cfg>(
        &self,
        ctx: &'cfg ConfigDataContext,
    ) -> &'cfg Location {
        &ctx.locations[ctx.delivery_routes[self.0[0]].src_location]
    }

    pub fn dst_location<'cfg>(
        &self,
        ctx: &'cfg ConfigDataContext,
    ) -> &'cfg Location {
        &ctx.locations
            [ctx.delivery_routes[self.0[self.0.len() - 1]].dst_location]
    }
}

pub struct DeliveryPipes(HashMap<DeliveryPipeId, DeliveryPipe>);

impl Index<DeliveryPipeId> for DeliveryPipes {
    type Output = DeliveryPipe;
    fn index(&self, index: DeliveryPipeId) -> &Self::Output {
        &self.0[&index]
    }
}

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ProductionLineId(u32);

pub struct ProductionLine {
    pub item: Item,
    pub blueprint: Blueprint,
    pub job_kind: JobKind,
    pub job_tax_rate: f64,
    pub export_kind: ProductionLineExportKind,
    pub export_dst_pipe: DeliveryPipeId,
    pub import_src_market_pipes: Vec<DeliveryPipeId>, // index into Locations
    pub import_src_production_lines: HashMap<TypeId, ProductionLineId>,
    pub location: LocationId,
    pub decryptor: Option<TypeId>,
}

impl ProductionLine {
    pub fn location<'cfg>(
        &self,
        cfg_ctx: &'cfg ConfigDataContext,
    ) -> &'cfg Location {
        &cfg_ctx.locations[self.location]
    }

    pub fn cost_efficiency(&self) -> f64 {
        unimplemented!()
    }

    pub fn max_slot_usage(&self) -> SlotCount {
        unimplemented!()
    }

    pub fn time_per_run(&self) -> Duration {
        unimplemented!()
    }

    // (runs per sequence, number of sequences)
    pub fn num_sequence_runs(&self, ctx: &ConfigDataContext) -> (u32, u32) {
        let time_per_run: Duration = self.time_per_run();
        match self.blueprint.kind() {
            BlueprintKind::BPO => {
                let num_runs = ctx.max_time.as_secs() / time_per_run.as_secs();
                (num_runs.try_into().unwrap(), 1)
            }
            BlueprintKind::BPC => {
                let num_runs = self.blueprint.runs;
                let time_per_sequence = self.time_per_run() * num_runs as u32;
                if time_per_run > ctx.max_time {
                    panic!("time_per_run > ctx.max_time");
                }
                let flexed_time_per_sequence =
                    time_per_sequence + ctx.daily_flex_time;
                // enforce flexed_time_per_sequence to be a multiple of 24 hours
                let final_time_per_sequence = flexed_time_per_sequence
                    + Duration::from_secs(24 * 60 * 60)
                    - Duration::new(
                        flexed_time_per_sequence.as_secs() % (24 * 60 * 60),
                        flexed_time_per_sequence.subsec_nanos(),
                    );
                let num_sequences =
                    ctx.max_time.as_secs() / final_time_per_sequence.as_secs();
                (num_runs.into(), num_sequences.try_into().unwrap())
            }
        }
    }

    pub fn num_produced(&self, ctx: &ConfigDataContext) -> u64 {
        unimplemented!()
    }

    pub fn num_runs(&self, ctx: &ConfigDataContext) -> u32 {
        let (runs_per_sequence, num_sequences) = self.num_sequence_runs(ctx);
        runs_per_sequence * num_sequences
    }

    pub fn sub_lines<'cfg>(
        &'cfg self,
        cfg_ctx: &'cfg ConfigDataContext,
    ) -> impl Iterator<Item = &'cfg ProductionLine> {
        self.import_src_production_lines
            .iter()
            .map(|(_, &production_line)| {
                &cfg_ctx.production_lines[production_line]
            })
    }

    pub fn src_markets_with_delivery_rates<'cfg>(
        &'cfg self,
        cfg_ctx: &'cfg ConfigDataContext,
    ) -> impl Iterator<Item = (&'cfg Location, DeliveryRate)> {
        self.import_src_market_pipes.iter().map(move |&pipe_id| {
            let pipe = &cfg_ctx.delivery_pipes[pipe_id];
            let location = pipe.src_location(cfg_ctx);
            let rate = pipe.rate(cfg_ctx);
            (location, rate)
        })
    }
}

pub struct ProductionLines(HashMap<ProductionLineId, ProductionLine>);

impl Index<ProductionLineId> for ProductionLines {
    type Output = ProductionLine;
    fn index(&self, index: ProductionLineId) -> &Self::Output {
        &self.0[&index]
    }
}

// Utility Data
