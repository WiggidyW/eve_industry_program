use crate::config::{self, Item};
use serde::Deserialize;
use std::{collections::HashMap, fs::File};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MarketOrder {
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeMarketOrders {
    pub orders: Vec<MarketOrder>, // cheapest first
    pub total: f64,
}

pub struct Api {
    pub adjusted_prices: HashMap<u32, f64>,
    pub cost_indices: HashMap<u32, config::ManufacturingValue>,
    pub market_orders: HashMap<u64, HashMap<u32, TypeMarketOrders>>,
    pub assets: HashMap<u64, HashMap<Item, i64>>,
}

impl Api {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let adjusted_prices = read_adjusted_prices()?;
        let cost_indices = read_cost_indices()?;
        let market_orders = read_market_orders()?;
        let assets = read_assets()?;
        Ok(Self {
            adjusted_prices,
            cost_indices,
            market_orders,
            assets,
        })
    }
}

fn read_adjusted_prices(
) -> Result<HashMap<u32, f64>, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(File::open(
        "adjusted_prices.json",
    )?)?)
}

fn read_cost_indices(
) -> Result<HashMap<u32, config::ManufacturingValue>, Box<dyn std::error::Error>>
{
    Ok(serde_json::from_reader(File::open("cost_indices.json")?)?)
}

fn read_market_orders() -> Result<
    HashMap<u64, HashMap<u32, TypeMarketOrders>>,
    Box<dyn std::error::Error>,
> {
    Ok(serde_json::from_reader(File::open("market_orders.json")?)?)
}

fn read_assets(
) -> Result<HashMap<u64, HashMap<Item, i64>>, Box<dyn std::error::Error>> {
    let DeserializedAssets(assets) =
        serde_json::from_reader(File::open("assets.json")?)?;
    let mut result = HashMap::new();
    for (location_id, deserialized_assets) in assets {
        let mut location_assets = HashMap::new();
        for deserialized_asset in deserialized_assets {
            location_assets.insert(
                Item {
                    type_id: deserialized_asset.type_id,
                    runs: deserialized_asset.runs,
                    me: deserialized_asset.me,
                    te: deserialized_asset.te,
                },
                deserialized_asset.quantity,
            );
        }
        result.insert(location_id, location_assets);
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct DeserializedAssets(HashMap<u64, Vec<DeserializedAsset>>);

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct DeserializedAsset {
    type_id: u32,
    runs: i16,
    me: i8,
    te: i8,
    quantity: i64,
}
