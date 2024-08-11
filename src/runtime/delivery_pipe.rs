use super::*;
use crate::config::{self, Item};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    iter,
    rc::Rc,
};

pub struct DeliveryPipe<'cfg, 'db, 'api> {
    pub routes: Vec<Rc<DeliveryRoute<'cfg, 'db, 'api>>>,
    pub deliveries: RefCell<HashMap<Item, i64>>,
}

impl<'cfg, 'db, 'api> DeliveryPipe<'cfg, 'db, 'api> {
    pub fn new(routes: Vec<Rc<DeliveryRoute<'cfg, 'db, 'api>>>) -> Self {
        Self {
            routes,
            deliveries: RefCell::new(HashMap::new()),
        }
    }

    pub fn src(&self) -> &Location<'cfg, 'db, 'api> {
        &self.routes[0].src
    }

    pub fn dst(&self) -> &Location<'cfg, 'db, 'api> {
        &self.routes[self.routes.len() - 1].dst
    }

    pub fn delivery_rate(&self) -> config::DeliveryRate {
        let mut rate = config::DeliveryRate::new();
        for route in self.routes.iter() {
            rate.collateral_rate += route.delivery_rate().collateral_rate;
            rate.m3_rate += route.delivery_rate().m3_rate;
        }
        rate
    }

    pub fn locations(
        &self,
    ) -> impl Iterator<Item = &Location<'cfg, 'db, 'api>> {
        self.routes
            .iter()
            .map(|route| route.src.as_ref())
            .chain(iter::once(self.dst()))
    }

    pub fn dst_locations(
        &self,
    ) -> impl Iterator<Item = &Location<'cfg, 'db, 'api>> {
        self.routes.iter().map(|route| route.dst.as_ref())
    }

    pub fn deliver(&self, item: Item, volume: i64) {
        *self.deliveries.borrow_mut().entry(item).or_insert(0) += volume;
    }

    pub fn deliveries(&self) -> DeliveryPipeDeliveries {
        DeliveryPipeDeliveries {
            inner: self.deliveries.borrow(),
        }
    }
}

pub struct DeliveryPipeDeliveries<'dp> {
    inner: Ref<'dp, HashMap<Item, i64>>,
}

impl<'dp> DeliveryPipeDeliveries<'dp> {
    pub fn iter(&self) -> impl Iterator<Item = (Item, i64)> + '_ {
        self.inner.iter().map(|(&item, &quantity)| (item, quantity))
    }
}
