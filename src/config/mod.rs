use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

mod delivery_route;
mod industry_slots;
mod item;
mod location;
mod production_line;

pub use delivery_route::*;
pub use industry_slots::*;
pub use item::*;
pub use location::*;
pub use production_line::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub locations: Vec<Location>,
    pub skills: HashMap<u32, u8>,
    pub slots: IndustrySlots,
    pub max_time: Duration, // time that all production lines are running for
    pub daily_flex_time: Duration, // extra time required for daily startables under 24 hours
    pub min_profit: f64,
    pub min_margin: f64,
}

impl Config {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_yaml::from_reader(std::fs::File::open(
            "config.yaml",
        )?)?)
    }
}
