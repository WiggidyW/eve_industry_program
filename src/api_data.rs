use crate::common::*;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    rc::Rc,
};

pub struct ApiDataContext {
    pub market_orders: MarketOrders,
    pub cost_indices: CostIndices,
    pub assets: Assets,
    pub adjusted_prices: AdjustedPrices,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketOrder {
    pub price: f64,
    pub volume: u64,
}

#[derive(Debug, Clone)]
pub struct TypeMarketOrders {
    orders: Rc<Vec<MarketOrder>>, // cheapest first
    reserved: u64,
    total: u64,
}

impl TypeMarketOrders {
    fn next_available(&self) -> Option<MarketOrder> {
        let mut current = 0;
        for market_order in self.orders.as_ref() {
            current += market_order.volume;
            if current > self.reserved {
                return Some(MarketOrder {
                    price: market_order.price,
                    volume: market_order.volume - (current - self.reserved),
                });
            }
        }
        None
    }

    fn min_sell(&self) -> Option<f64> {
        self.orders.as_ref().first().map(|order| order.price)
    }

    fn reserve(&mut self, volume: u64) {
        self.reserved += volume;
    }
}

#[derive(Debug, Clone)]
pub struct LocationMarketOrders(HashMap<TypeId, TypeMarketOrders>);

impl LocationMarketOrders {
    fn next_available(&self, type_id: TypeId) -> Option<MarketOrder> {
        self.0
            .get(&type_id)
            .and_then(|type_market_orders| type_market_orders.next_available())
    }

    fn min_sell(&self, type_id: TypeId) -> Option<f64> {
        self.0
            .get(&type_id)
            .and_then(|type_market_orders| type_market_orders.min_sell())
    }

    fn reserve(&mut self, type_id: TypeId, volume: u64) {
        self.0.get_mut(&type_id).unwrap().reserve(volume);
    }
}

#[derive(Debug, Clone)]
pub struct MarketOrders(HashMap<LocationId, LocationMarketOrders>);

impl MarketOrders {
    pub fn next_available(
        &self,
        location_id: LocationId,
        type_id: TypeId,
    ) -> Option<MarketOrder> {
        self.0.get(&location_id).and_then(|location_market_orders| {
            location_market_orders.next_available(type_id)
        })
    }

    pub fn min_sell(
        &self,
        location_id: LocationId,
        type_id: TypeId,
    ) -> Option<f64> {
        self.0.get(&location_id).and_then(|location_market_orders| {
            location_market_orders.min_sell(type_id)
        })
    }

    pub fn reserve(
        &mut self,
        location_id: LocationId,
        type_id: TypeId,
        volume: u64,
    ) {
        self.0
            .get_mut(&location_id)
            .unwrap()
            .reserve(type_id, volume);
    }
    // pub fn get_mut(
    //     &mut self,
    //     location_id: LocationId,
    // ) -> &mut LocationMarketOrders {
    //     self.0
    //         .entry(location_id)
    //         .or_insert(LocationMarketOrders(HashMap::new()))
    // }
}

// gonna need another struct for temporary reserves ^ when calcing profit inclusive of intermediate prod lines

pub struct SystemCostIndices {
    pub manufacturing: f64,
    pub copying: f64,
    pub invention: f64,
    pub reaction: f64,
}

impl Index<JobKind> for SystemCostIndices {
    type Output = f64;
    fn index(&self, job_kind: JobKind) -> &Self::Output {
        match job_kind {
            JobKind::Manufacturing => &self.manufacturing,
            JobKind::Copying => &self.copying,
            JobKind::Invention => &self.invention,
            JobKind::Reaction => &self.reaction,
        }
    }
}

pub struct CostIndices(HashMap<SystemId, SystemCostIndices>);

impl Index<SystemId> for CostIndices {
    type Output = SystemCostIndices;
    fn index(&self, system_id: SystemId) -> &Self::Output {
        self.0.get(&system_id).unwrap_or(&SystemCostIndices {
            manufacturing: 0.0,
            copying: 0.0,
            invention: 0.0,
            reaction: 0.0,
        })
    }
}

pub struct LocationAssets(HashMap<LocationId, (Item, u64)>);

pub struct Assets(HashMap<LocationId, LocationAssets>);

pub struct AdjustedPrices(HashMap<TypeId, f64>);

impl Index<TypeId> for AdjustedPrices {
    type Output = f64;
    fn index(&self, type_id: TypeId) -> &Self::Output {
        self.0.get(&type_id).unwrap_or(&0.0)
    }
}
