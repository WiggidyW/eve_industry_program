use std::collections::HashMap;

use crate::{common::*, config_data::*, static_data::*};

pub struct Deliveries(HashMap<usize, DeliveryRouteDeliveries>);

pub struct DeliveryRouteDeliveries(HashMap<Item, u64>);

pub struct ProfitCheckData {
    pub materials: HashMap<Item, u64>,
}
