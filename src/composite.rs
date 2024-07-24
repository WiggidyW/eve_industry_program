use std::collections::HashMap;

use crate::{api_data::*, common::*, config_data::*, static_data::*};

pub fn per_run_installation_cost(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
) -> f64 {
    let materials = per_run_raw_job_materials(
        production_line.item.type_id(),
        production_line.job_kind,
        None, // decryptor doesn't matter for installation cost
    );
    let mut eiv = 0.0;
    for (type_id, quantity) in materials {
        let adjusted_price = api_ctx.adjusted_prices[type_id];
        eiv += adjusted_price * quantity as f64;
    }
    let system_cost_indices =
        &api_ctx.cost_indices[production_line.location(cfg_ctx).system_id];
    let cost_index = system_cost_indices[production_line.job_kind];
    let cost_efficiency = production_line.cost_efficiency();
    let job_kind_multiplier =
        production_line.job_kind.installation_cost_multiplier();
    let base_cost = eiv * cost_index * cost_efficiency * job_kind_multiplier;
    base_cost + (base_cost * production_line.job_tax_rate)
}

fn per_run_raw_job_materials_with_me(
    production_line: &ProductionLine,
) -> impl Iterator<Item = (TypeId, f64)> {
    let cost_efficiency = production_line.cost_efficiency();
    per_run_raw_job_materials(
        production_line.item.type_id(),
        production_line.job_kind,
        production_line.decryptor,
    )
    .into_iter()
    .map(move |(type_id, quantity)| {
        if quantity > 1 {
            (type_id, quantity as f64 * cost_efficiency)
        } else {
            (type_id, quantity as f64)
        }
    })
}

fn raw_job_materials_with_me(
    production_line: &ProductionLine,
    // cfg_ctx: &ConfigDataContext,
    num_runs: f64,
) -> impl Iterator<Item = (TypeId, f64)> {
    // let num_runs: f64 = production_line.num_runs(cfg_ctx).try_into().unwrap();
    per_run_raw_job_materials_with_me(production_line)
        .map(move |(type_id, quantity)| (type_id, (quantity * num_runs).ceil()))
}

pub fn job_materials_market_provided<'cfg>(
    production_line: &'cfg ProductionLine,
    num_runs: f64,
) -> impl Iterator<Item = (TypeId, f64)> + 'cfg {
    raw_job_materials_with_me(production_line, num_runs).filter_map(
        |(type_id, quantity)| {
            if production_line
                .import_src_production_lines
                .contains_key(&type_id)
            {
                None
            } else {
                Some((type_id, quantity))
            }
        },
    )
}

pub fn job_materials_sub_line_provided<'cfg>(
    production_line: &'cfg ProductionLine,
    cfg_ctx: &'cfg ConfigDataContext,
    num_runs: f64,
) -> impl Iterator<Item = (Item, f64)> + 'cfg {
    raw_job_materials_with_me(production_line, num_runs)
        .filter_map(|(type_id, quantity)| {
            production_line
                .import_src_production_lines
                .get(&type_id)
                .map(|&src_line_id| {
                    (cfg_ctx.production_lines[src_line_id].item, quantity)
                })
        })
        .chain(
            match production_line.blueprint.kind() {
                BlueprintKind::BPO => None,
                BlueprintKind::BPC => {
                    let bpc = Item::Blueprint(production_line.blueprint);
                    let (_, num_bpcs_for_full_run) =
                        production_line.num_sequence_runs(cfg_ctx);
                    let full_run_num_runs = production_line.num_runs(cfg_ctx);
                    let num_bpcs_per_run =
                        num_bpcs_for_full_run as f64 / full_run_num_runs as f64;
                    Some((bpc, num_bpcs_per_run))
                }
            }
            .into_iter(),
        )
}

pub fn market_cost_with_delivery(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
    mut market_orders: MarketOrders,
    materials: impl Iterator<Item = (TypeId, f64)>,
) -> Option<(f64, MarketOrders)> {
    let src_markets = production_line
        .src_markets_with_delivery_rates(cfg_ctx)
        .collect::<Vec<_>>();
    let mut cost: f64 = 0.0;
    for (type_id, quantity) in materials {
        let type_volume = type_volume(type_id);
        let mut reserved: f64 = 0.0;
        let mut type_cost: f64 = 0.0;
        while reserved.floor() < quantity {
            let mut cheapest_market: Option<LocationId> = None;
            let mut cheapest_reservable = 0;
            let mut cheapest_price_with_delivery = f64::INFINITY;
            for (location, rate) in &src_markets {
                if let Some(order) =
                    market_orders.next_available(location.id, type_id)
                {
                    let price_with_delivery = order.price
                        + rate.m3_rate * type_volume
                        + rate.collateral_rate * order.price;
                    if price_with_delivery < cheapest_price_with_delivery {
                        cheapest_market = Some(location.id);
                        cheapest_reservable = order.volume;
                        cheapest_price_with_delivery = price_with_delivery;
                    }
                }
            }
            if let Some(location_id) = cheapest_market {
                let cheapest_reserve = cheapest_reservable
                    .min((quantity - reserved.floor()) as u64);
                reserved += cheapest_reserve as f64;
                type_cost +=
                    cheapest_price_with_delivery * cheapest_reserve as f64;
                market_orders.reserve(location_id, type_id, cheapest_reserve);
            }
        }
        type_cost += (type_cost / reserved) * (quantity - reserved);
        cost += type_cost;
    }
    Some((cost, market_orders))
}

pub fn market_revenue_with_delivery(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
) -> f64 {
    let dst_pipe = &cfg_ctx.delivery_pipes[production_line.export_dst_pipe];
    let dst_location = dst_pipe.dst_location(cfg_ctx);
    let rate = dst_pipe.rate(cfg_ctx);
    let type_volume = type_volume(production_line.item.type_id());
    let per_run_quantity = per_run_product_quantity(
        production_line.item.type_id(),
        production_line.job_kind,
        production_line.decryptor,
    );
    let quantity = per_run_quantity * production_line.num_runs(cfg_ctx) as f64;
    let price = api_ctx
        .market_orders
        .min_sell(dst_location.id, production_line.item.type_id())
        .unwrap_or(0.0);
    quantity
        * (price
            - (price * dst_location.unwrap_market().broker_fee)
            - (price * dst_location.unwrap_market().sales_tax)
            - (rate.m3_rate * type_volume)
            - (rate.collateral_rate * price))
}

pub fn profit(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
) -> Option<f64> {
    let num_produced = production_line.num_produced(cfg_ctx);
    let revenue =
        market_revenue_with_delivery(production_line, cfg_ctx, api_ctx);
    let cost = match cost_by_num_produced(
        production_line,
        cfg_ctx,
        api_ctx,
        api_ctx.market_orders.clone(),
        production_line.num_produced(cfg_ctx) as f64,
    ) {
        Some((cost, _)) => cost,
        None => return None,
    };
    Some(revenue - cost)
}

fn cost_by_num_produced(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
    market_orders: MarketOrders,
    num_produced: f64,
) -> Option<(f64, MarketOrders)> {
    let full_num_runs = production_line.num_runs(cfg_ctx) as f64;
    let full_num_produced = production_line.num_produced(cfg_ctx) as f64;
    let num_runs = num_produced * (full_num_runs / full_num_produced);
    let installation_cost =
        per_run_installation_cost(production_line, cfg_ctx, api_ctx) * num_runs;
    let (market_cost_with_delivery, mut market_orders) =
        match market_cost_with_delivery(
            production_line,
            cfg_ctx,
            api_ctx,
            market_orders,
            job_materials_market_provided(production_line, num_runs),
        ) {
            Some((mcwd, mo)) => (mcwd, mo),
            None => return None,
        };
    let mut cost = installation_cost + market_cost_with_delivery;
    for (item, sub_num_produced) in
        job_materials_sub_line_provided(production_line, cfg_ctx, num_runs)
    {
        match cost_by_num_produced(
            &cfg_ctx.production_lines
                [production_line.import_src_production_lines[&item.type_id()]],
            cfg_ctx,
            api_ctx,
            market_orders,
            sub_num_produced,
        ) {
            Some((sub_cost, mo)) => {
                cost += sub_cost;
                market_orders = mo;
            }
            None => return None,
        };
    }
    Some((cost, market_orders))
}
