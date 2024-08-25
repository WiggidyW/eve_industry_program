use super::*;
use crate::api_data;
use crate::config;
use crate::industry_db;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct TypeMarketOrders<'api> {
    pub inner: &'api api_data::TypeMarketOrders,
    pub reserved: RefCell<f64>, // committed reservations
    reserved_current: RefCell<f64>, // current profit check reservations
    reserved_current_context: RefCell<u64>, // current profit check context
}

impl<'api> TypeMarketOrders<'api> {
    pub fn new(inner: &'api api_data::TypeMarketOrders) -> Self {
        Self {
            inner,
            reserved: RefCell::new(0.0),
            reserved_current: RefCell::new(0.0),
            reserved_current_context: RefCell::new(0),
        }
    }

    fn reserved_current(&self, context: Option<u64>) -> f64 {
        match context {
            Some(context)
                if *self.reserved_current_context.borrow() == context =>
            {
                *self.reserved_current.borrow()
            }
            Some(context) => {
                *self.reserved_current_context.borrow_mut() = context;
                *self.reserved_current.borrow_mut() = 0.0;
                0.0
            }
            None => 0.0,
        }
    }

    fn reserved(&self, context: Option<u64>) -> f64 {
        *self.reserved.borrow() + self.reserved_current(context)
    }

    pub fn reserve(&self, context: Option<u64>, volume: f64) {
        match context {
            Some(context)
                if *self.reserved_current_context.borrow() == context =>
            {
                *self.reserved_current.borrow_mut() += volume;
            }
            Some(context) => {
                *self.reserved_current_context.borrow_mut() = context;
                *self.reserved_current.borrow_mut() = volume;
            }
            None => {
                *self.reserved.borrow_mut() += volume;
            }
        }
    }

    pub fn next_available(
        &self,
        context: Option<u64>,
    ) -> Option<api_data::MarketOrder> {
        let mut current = 0.0;
        let reserved = self.reserved(context);
        if reserved >= self.inner.total {
            return None;
        }
        for order in &self.inner.orders {
            current += order.volume;
            if current > reserved {
                return Some(api_data::MarketOrder {
                    price: order.price,
                    volume: current - reserved,
                });
            }
        }
        None
    }

    pub fn num_purchased(&self, context: Option<u64>) -> i64 {
        (*self.reserved.borrow() + self.reserved_current(context)) as i64
    }

    pub fn num_purchased_with_stats(
        &self,
        context: Option<u64>,
        cost: &mut f64,
    ) -> (i64, PurchaseStats) {
        let reserved = self.reserved(context);

        if reserved == 0.0 {
            return (
                0,
                PurchaseStats {
                    price_low: 0.0,
                    price_high: 0.0,
                },
            );
        }

        let num_purchased = reserved as i64;
        let mut stats = PurchaseStats {
            price_low: self.inner.orders[0].price,
            price_high: self.inner.orders[0].price,
        };

        let mut current = 0.0;
        for order in &self.inner.orders {
            stats.price_high = order.price;
            current += order.volume;
            if current >= reserved {
                *cost += order.price * (order.volume - (current - reserved));
                break;
            } else {
                *cost += order.price * order.volume;
            }
        }

        (num_purchased, stats)
    }

    pub fn min_sell(&self) -> Option<f64> {
        self.inner.orders.first().map(|order| order.price)
    }

    pub fn total_volume(&self) -> f64 {
        self.inner.total
    }
}

pub struct PurchaseStats {
    pub price_low: f64,
    pub price_high: f64,
}

pub struct LocationMarketOrders<'api> {
    pub inner: HashMap<u32, TypeMarketOrders<'api>>,
}

impl<'api> LocationMarketOrders<'api> {
    pub fn iter_purchases<'this, 'c>(
        &'this self,
        context: Option<u64>,
        cost: &'c mut f64,
    ) -> impl Iterator<Item = (u32, i64, PurchaseStats)> + 'this + 'c
    where
        'api: 'c,
        'c: 'this,
        'this: 'c,
    {
        self.inner
            .iter()
            .map(move |(&type_id, orders)| {
                (type_id, orders.num_purchased_with_stats(context, cost))
            })
            .filter(|(_, (num_purchased, _))| *num_purchased > 0)
            .map(move |(type_id, (num_purchased, stats))| {
                (type_id, num_purchased, stats)
            })
    }
    pub fn new(
        inner: Option<&'api HashMap<u32, api_data::TypeMarketOrders>>,
    ) -> Self {
        Self {
            inner: inner
                .map(|inner| {
                    inner
                        .iter()
                        .map(|(&type_id, orders)| {
                            (type_id, TypeMarketOrders::new(orders))
                        })
                        .collect()
                })
                .unwrap_or(HashMap::new()),
        }
    }

    pub fn total_volume(&self, type_id: &u32) -> f64 {
        self.inner
            .get(type_id)
            .map(|orders| orders.total_volume())
            .unwrap_or(0.0)
    }

    pub fn reserve(&self, context: Option<u64>, type_id: &u32, volume: f64) {
        self.inner.get(type_id).unwrap().reserve(context, volume);
    }

    pub fn reserve_i64(
        &self,
        context: Option<u64>,
        type_id: &u32,
        volume: i64,
    ) {
        self.inner
            .get(type_id)
            .unwrap()
            .reserve(context, volume as f64);
    }

    pub fn next_available(
        &self,
        context: Option<u64>,
        type_id: &u32,
    ) -> Option<api_data::MarketOrder> {
        self.inner
            .get(&type_id)
            .and_then(|orders| orders.next_available(context))
    }

    pub fn min_sell(&self, type_id: &u32) -> Option<f64> {
        self.inner
            .get(type_id)
            .map(|orders| orders.min_sell())
            .flatten()
    }

    pub fn num_purchased(&self, context: Option<u64>, type_id: &u32) -> i64 {
        self.inner
            .get(type_id)
            .map(|orders| orders.num_purchased(context))
            .unwrap_or(0)
    }
}
