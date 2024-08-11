mod api_data;
mod composite;
mod config;
mod industry_db;
mod runtime;

mod error;
use error::Error;

#[tokio::main]
async fn main() {
    let db = industry_db::new_industry_database().await.unwrap();
    let cfg = config::Config::read().unwrap();
    let api = api_data::Api::read().unwrap();

    let db_lines = composite::get_db_lines(
        cfg.locations.iter(),
        &cfg.skills,
        cfg.max_time,
        &db,
    )
    .await
    .unwrap();
    let (type_volumes, type_names) = composite::get_db_volumes_and_names(
        cfg.locations.iter(),
        db_lines.values(),
        &db,
    )
    .await
    .unwrap();

    let mut runtime = runtime::RuntimeData::new(
        &cfg.locations,
        &cfg.slots,
        cfg.max_time,
        cfg.daily_flex_time,
        cfg.min_profit,
        cfg.min_margin,
        &db_lines,
        &type_volumes,
        &api.adjusted_prices,
        &api.cost_indices,
        &api.market_orders,
        &api.assets,
    );

    runtime.build();
    runtime.write(&type_names).unwrap();
}
