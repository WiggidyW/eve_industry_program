use std::collections::HashMap;

use crate::{
    api_data::*, common::*, config_data::*, run_data::*, static_data::*,
};

fn per_run_installation_cost(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
) -> f64 {
    let materials = per_run_raw_job_materials(
        production_line.blueprint,
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
    let material_efficiency = production_line.material_efficiency();
    per_run_raw_job_materials(
        production_line.blueprint,
        production_line.item.type_id(),
        production_line.job_kind,
        production_line.decryptor,
    )
    .into_iter()
    .map(move |(type_id, quantity)| {
        if quantity > 1 {
            (type_id, quantity as f64 * material_efficiency)
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
        .map(move |(type_id, quantity)| (type_id, quantity * num_runs))
}

fn ceil_materials<'cfg>(
    iter: impl Iterator<Item = (Item, f64)> + 'cfg,
) -> impl Iterator<Item = (Item, f64)> + 'cfg {
    iter.map(|(item, quantity)| (item, quantity.ceil()))
}

fn job_materials_market_provided<'cfg>(
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

fn ceil_job_materials_market_provided<'cfg>(
    production_line: &'cfg ProductionLine,
    num_runs: f64,
) -> impl Iterator<Item = (Item, f64)> + 'cfg {
    ceil_materials(
        job_materials_market_provided(production_line, num_runs)
            .map(|(type_id, quantity)| (Item::Item(type_id), quantity)),
    )
}

fn job_materials_sub_line_provided<'cfg>(
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
                    let num_bpcs = num_runs * num_bpcs_per_run;
                    Some((bpc, num_bpcs))
                }
            }
            .into_iter(),
        )
}

fn ceil_job_materials_sub_line_provided<'cfg>(
    production_line: &'cfg ProductionLine,
    cfg_ctx: &'cfg ConfigDataContext,
    num_runs: f64,
) -> impl Iterator<Item = (Item, f64)> + 'cfg {
    ceil_materials(job_materials_sub_line_provided(
        production_line,
        cfg_ctx,
        num_runs,
    ))
}

fn ceil_job_materials<'cfg>(
    production_line: &'cfg ProductionLine,
    cfg_ctx: &'cfg ConfigDataContext,
) -> impl Iterator<Item = (Item, f64)> + 'cfg {
    let num_runs = production_line.num_runs(cfg_ctx);
    let market_provided = ceil_materials(
        job_materials_market_provided(production_line, num_runs as f64)
            .map(|(type_id, quantity)| (Item::Item(type_id), quantity)),
    );
    let sub_line_provided = ceil_materials(job_materials_sub_line_provided(
        production_line,
        cfg_ctx,
        num_runs as f64,
    ));
    market_provided.chain(sub_line_provided)
}

fn market_cost_with_delivery(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    // api_ctx: &ApiDataContext,
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
        while reserved < quantity {
            let mut cheapest_market: Option<LocationId> = None;
            let mut cheapest_reservable = 0.0;
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
                let cheapest_reserve =
                    (cheapest_reservable as f64).min(quantity - reserved);
                let cheapest_reserve_whole = cheapest_reserve.floor();
                reserved += cheapest_reserve;
                type_cost += cheapest_price_with_delivery * cheapest_reserve;
                market_orders.reserve(
                    location_id,
                    type_id,
                    cheapest_reserve_whole,
                );
            } else {
                return None;
            }
        }
        type_cost += (type_cost / reserved) * (quantity - reserved);
        cost += type_cost;
    }
    Some((cost, market_orders))
}

fn market_revenue_with_delivery(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    market_orders: &MarketOrders,
) -> f64 {
    let dst_pipe = &cfg_ctx.delivery_pipes[production_line.export_dst_pipe];
    let dst_location = dst_pipe.dst_location(cfg_ctx);
    let rate = dst_pipe.rate(cfg_ctx);
    let type_volume = type_volume(production_line.item.type_id());
    let per_run_quantity = per_run_product_quantity(
        production_line.blueprint,
        production_line.item.type_id(),
        production_line.job_kind,
        production_line.decryptor,
    );
    let quantity = per_run_quantity * production_line.num_runs(cfg_ctx) as f64;
    let price = market_orders
        .min_sell(dst_location.id, production_line.item.type_id())
        .unwrap_or(0.0);
    quantity
        * (price
            - (price * dst_location.unwrap_market().broker_fee)
            - (price * dst_location.unwrap_market().sales_tax)
            - (rate.m3_rate * type_volume)
            - (rate.collateral_rate * price))
}

fn profit(
    production_line: &ProductionLine,
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
    market_orders: &MarketOrders,
) -> Option<f64> {
    let revenue =
        market_revenue_with_delivery(production_line, cfg_ctx, market_orders);
    let cost = match cost_by_num_produced(
        production_line,
        cfg_ctx,
        api_ctx,
        market_orders.clone(),
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
            // api_ctx,
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

fn can_build(
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
    production_line_id: ProductionLineId,
    production_line: &ProductionLine,
    stock_targets: &Assets,
    deliveries: &Deliveries,
    builds: &Builds,
    slots: &SlotCount,
    // market_orders: &mut MarketOrders,
) -> bool {
    if !slots.can_fit(&production_line.max_slot_usage(cfg_ctx)) {
        false
    } else if builds.get(production_line_id) >= production_line.parallel {
        false
    } else {
        match production_line.export_kind {
            ProductionLineExportKind::Intermediate => {
                let num_stored = api_ctx
                    .assets
                    .get_amount(production_line.location, production_line.item);
                let num_exported = deliveries.get_amount(
                    production_line.export_dst_pipe,
                    production_line.item,
                );
                let num_needed = stock_targets
                    .get_amount(production_line.location, production_line.item);
                num_stored >= num_needed + num_exported
            }
            ProductionLineExportKind::Product => {
                let num_stored = api_ctx.assets.get_amount(
                    production_line
                        .export_dst_pipe(cfg_ctx)
                        .dst_location_id(cfg_ctx),
                    production_line.item,
                );
                let num_built = builds.get(production_line_id) as f64
                    * production_line.num_produced(cfg_ctx);
                let num_needed = stock_targets.get_amount(
                    production_line
                        .export_dst_pipe(cfg_ctx)
                        .dst_location_id(cfg_ctx),
                    production_line.item,
                );
                num_stored + num_built >= num_needed
            }
        }
    }
}

fn build(
    cfg_ctx: &ConfigDataContext,
    production_line_id: ProductionLineId,
    production_line: &ProductionLine,
    deliveries: &mut Deliveries,
    builds: &mut Builds,
    slots: &mut SlotCount,
    market_orders: &mut MarketOrders,
) {
    do_sub_line_imports(cfg_ctx, production_line, deliveries);
    do_market_imports(cfg_ctx, production_line, deliveries, market_orders);
    if production_line.product() {
        do_market_export(cfg_ctx, production_line, deliveries);
    }
    slots.reserve(production_line.job_kind);
    builds.add(production_line_id);
}

// call until it returns false
// remember, intermediates never create export deliveries
fn try_build_intermediates(
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
    stock_targets: &Assets,
    deliveries: &mut Deliveries,
    builds: &mut Builds,
    slots: &mut SlotCount,
    market_orders: &mut MarketOrders,
) -> bool {
    let mut any_built = false;
    for (&id, production_line) in &cfg_ctx.production_lines {
        if !production_line.intermediate() {
            continue;
        }
        if !can_build(
            cfg_ctx,
            api_ctx,
            id,
            production_line,
            stock_targets,
            deliveries,
            builds,
            slots,
        ) {
            continue;
        }
        build(
            cfg_ctx,
            id,
            production_line,
            deliveries,
            builds,
            slots,
            market_orders,
        );
        any_built = true;
    }
    any_built
}

fn do_sub_line_imports(
    cfg_ctx: &ConfigDataContext,
    production_line: &ProductionLine,
    deliveries: &mut Deliveries,
) {
    for (item, quantity) in ceil_job_materials_sub_line_provided(
        production_line,
        cfg_ctx,
        production_line.num_runs(cfg_ctx) as f64,
    ) {
        let pipe_id = production_line
            .import_src_production_line_pipe_id(cfg_ctx, item.type_id());
        deliveries.make_delivery(pipe_id, item, quantity);
    }
}

fn do_market_export(
    cfg_ctx: &ConfigDataContext,
    production_line: &ProductionLine,
    deliveries: &mut Deliveries,
) {
    deliveries.make_delivery(
        production_line.export_dst_pipe,
        production_line.item,
        production_line.num_produced(cfg_ctx),
    );
}

fn do_market_imports(
    cfg_ctx: &ConfigDataContext,
    production_line: &ProductionLine,
    deliveries: &mut Deliveries,
    market_orders: &mut MarketOrders,
) {
    let src_markets = production_line
        .src_markets_with_delivery_pipes(cfg_ctx)
        .map(|(location, pipe_id, pipe)| {
            (location, pipe_id, pipe.rate(cfg_ctx))
        })
        .collect::<Vec<_>>();
    for (item, quantity) in ceil_job_materials_market_provided(
        production_line,
        production_line.num_runs(cfg_ctx) as f64,
    ) {
        let type_volume = type_volume(item.type_id());
        let mut reserved = 0.0;
        while reserved < quantity {
            let mut cheapest_market: Option<LocationId> = None;
            let mut cheapest_pipe_id: Option<DeliveryPipeId> = None;
            let mut cheapest_reservable = 0.0;
            let mut cheapest_price_with_delivery = f64::INFINITY;
            for (location, pipe_id, rate) in &src_markets {
                if let Some(order) =
                    market_orders.next_available(location.id, item.type_id())
                {
                    let price_with_delivery = order.price
                        + rate.m3_rate * type_volume
                        + rate.collateral_rate * order.price;
                    if price_with_delivery < cheapest_price_with_delivery {
                        cheapest_market = Some(location.id);
                        cheapest_pipe_id = Some(*pipe_id);
                        cheapest_reservable = order.volume;
                        cheapest_price_with_delivery = price_with_delivery;
                    }
                }
            }
            if let Some(location_id) = cheapest_market {
                let cheapest_reserve =
                    cheapest_reservable.min(quantity - reserved);
                reserved += cheapest_reserve;
                deliveries.make_delivery(
                    cheapest_pipe_id.unwrap(),
                    item,
                    cheapest_reserve,
                );
                market_orders.reserve(
                    location_id,
                    item.type_id(),
                    cheapest_reserve,
                );
            } else {
                let mut highest_volume_market: Option<LocationId> = None;
                let mut highest_volume_pipe_id: Option<DeliveryPipeId> = None;
                let mut highest_volume = 0.0;
                for (location, pipe_id, _) in &src_markets {
                    let volume =
                        market_orders.volume(location.id, item.type_id());
                    if volume > highest_volume {
                        highest_volume_market = Some(location.id);
                        highest_volume_pipe_id = Some(*pipe_id);
                        highest_volume = volume;
                    }
                }
                deliveries.make_delivery(
                    highest_volume_pipe_id.unwrap(),
                    item,
                    quantity - reserved,
                );
                market_orders.reserve(
                    highest_volume_market.unwrap(),
                    item.type_id(),
                    quantity - reserved,
                );
                reserved = quantity;
            }
        }
    }
}

pub fn stock_targets(cfg_ctx: &ConfigDataContext) -> Assets {
    let mut assets = Assets::new();
    for (_, production_line) in &cfg_ctx.production_lines {
        let num_runs = production_line.num_runs(cfg_ctx) as f64;
        for location_id in production_line
            .import_src_market_pipes(cfg_ctx)
            .map(|delivery_pipe| delivery_pipe.location_ids(cfg_ctx))
            .flatten()
        {
            for (item, quantity) in
                ceil_job_materials_market_provided(production_line, num_runs)
            {
                assets.add_amount(
                    location_id,
                    item,
                    quantity * production_line.parallel as f64,
                );
            }
        }
        for (item, quantity) in ceil_job_materials_sub_line_provided(
            production_line,
            cfg_ctx,
            num_runs,
        ) {
            for location_id in production_line
                .import_src_production_line_pipe(cfg_ctx, item.type_id())
                .location_ids(cfg_ctx)
            {
                assets.add_amount(
                    location_id,
                    item,
                    quantity * production_line.parallel as f64,
                );
            }
        }
        if production_line.product() {
            let quantity = production_line.num_produced(cfg_ctx);
            for location_id in production_line
                .export_dst_pipe(cfg_ctx)
                .location_ids(cfg_ctx)
            {
                assets.add_amount(
                    location_id,
                    production_line.item,
                    quantity * production_line.parallel as f64,
                );
            }
        }
    }
    assets
}

pub fn run(
    cfg_ctx: &ConfigDataContext,
    api_ctx: &ApiDataContext,
) -> (Deliveries, Builds, MarketOrders) {
    let stock_targets = stock_targets(cfg_ctx);
    let mut deliveries = Deliveries::new();
    let mut builds = Builds::new();
    let mut slots = cfg_ctx.slot_count.clone();
    let mut market_orders = api_ctx.market_orders.clone();
    loop {
        loop {
            if !try_build_intermediates(
                cfg_ctx,
                api_ctx,
                &stock_targets,
                &mut deliveries,
                &mut builds,
                &mut slots,
                &mut market_orders,
            ) {
                break;
            }
        }
        let mut highest_profit: Option<(ProductionLineId, f64)> = None;
        for (&id, production_line) in &cfg_ctx.production_lines {
            if can_build(
                cfg_ctx,
                api_ctx,
                id,
                production_line,
                &stock_targets,
                &deliveries,
                &builds,
                &slots,
            ) {
                match (
                    highest_profit,
                    profit(production_line, cfg_ctx, api_ctx, &market_orders),
                ) {
                    (Some((_, current_profit)), Some(profit))
                        if profit > current_profit =>
                    {
                        highest_profit = Some((id, profit))
                    }
                    (None, Some(profit)) => highest_profit = Some((id, profit)),
                    _ => {}
                }
            }
        }
        if let Some((id, _)) = highest_profit {
            build(
                cfg_ctx,
                id,
                &cfg_ctx.production_lines[id],
                &mut deliveries,
                &mut builds,
                &mut slots,
                &mut market_orders,
            );
        } else {
            break;
        }
    }
    (deliveries, builds, market_orders)
}
