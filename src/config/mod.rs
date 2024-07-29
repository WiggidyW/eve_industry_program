use crate::typedef::*;
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

pub struct Config {
    pub locations: HashMap<u64, Location>,
    pub skills: HashMap<u32, u8>,
    pub slots: IndustrySlots,
    pub max_time: Duration, // time that all production lines are running for
    pub daily_flex_time: Duration, // extra time required for daily startables under 24 hours
}
