use std::collections::HashMap;

use crate::{api_data::*, common::*, config_data::*, static_data::*};

pub struct Deliveries(HashMap<DeliveryPipeId, DeliveryRouteDeliveries>);

impl Deliveries {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_amount(
        &self,
        delivery_pipe_id: DeliveryPipeId,
        item: Item,
    ) -> u64 {
        self.0
            .get(&delivery_pipe_id)
            .and_then(|delivery_route_deliveries| {
                delivery_route_deliveries.0.get(&item)
            })
            .copied()
            .unwrap_or(0)
    }

    pub fn make_delivery(
        &mut self,
        delivery_pipe_id: DeliveryPipeId,
        item: Item,
        amount: u64,
    ) {
        self.0
            .entry(delivery_pipe_id)
            .or_insert_with(|| DeliveryRouteDeliveries(HashMap::new()))
            .0
            .entry(item)
            .and_modify(|current_amount| *current_amount += amount)
            .or_insert(amount);
    }

    pub fn get_available(
        &self,
        cfg_ctx: &ConfigDataContext,
        api_ctx: &ApiDataContext,
        location_id: LocationId,
        item: Item,
    ) -> u64 {
        let mut available = api_ctx.assets.get_amount(location_id, item);
        for &delivery_pipe_id in self.0.keys() {
            let delivery_amount = self.get_amount(delivery_pipe_id, item);
            if delivery_amount == 0 {
                continue;
            }
            let delivery_pipe = &cfg_ctx.delivery_pipes[delivery_pipe_id];
            if delivery_pipe.dst_location_id(cfg_ctx) == location_id {
                available += delivery_amount;
            } else if delivery_pipe
                .src_location_ids(cfg_ctx)
                .any(|src_location_id| location_id == src_location_id)
            {
                available -= delivery_amount;
            }
        }
        available
    }
}

pub struct DeliveryRouteDeliveries(HashMap<Item, u64>);

pub struct Builds(HashMap<ProductionLineId, u32>);

impl Builds {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, production_line_id: ProductionLineId) -> u32 {
        *self.0.get(&production_line_id).unwrap_or(&0)
    }

    pub fn add(&mut self, production_line_id: ProductionLineId) {
        self.0
            .entry(production_line_id)
            .and_modify(|current_amount| *current_amount += 1)
            .or_insert(1);
    }
}

pub struct ProfitCheckData {
    pub materials: HashMap<Item, u64>,
}

// pub struct
