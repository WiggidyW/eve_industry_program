use super::*;
use crate::config;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct DeliveryRoute<'cfg, 'db, 'api> {
    pub inner: &'cfg config::DeliveryRoute,
    pub src: Rc<Location<'cfg, 'db, 'api>>,
    pub dst: Rc<Location<'cfg, 'db, 'api>>,
    pub pipes: RefCell<Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>>,
}

impl<'cfg, 'db, 'api> DeliveryRoute<'cfg, 'db, 'api> {
    pub fn new(
        inner: &'cfg config::DeliveryRoute,
        src: Rc<Location<'cfg, 'db, 'api>>,
        dst: Rc<Location<'cfg, 'db, 'api>>,
    ) -> Self {
        Self {
            inner,
            src,
            dst,
            pipes: RefCell::new(Vec::new()),
        }
    }

    pub fn delivery_rate(&self) -> &config::DeliveryRate {
        &self.inner.rate
    }

    pub fn pipes(&self) -> DeliveryRoutePipes<'_, 'cfg, 'db, 'api> {
        DeliveryRoutePipes {
            inner: self.pipes.borrow(),
        }
    }
}

pub struct DeliveryRoutePipes<'dr, 'cfg, 'db, 'api> {
    inner: Ref<'dr, Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>>,
}

impl<'dr, 'cfg, 'db, 'api> DeliveryRoutePipes<'dr, 'cfg, 'db, 'api> {
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &Rc<DeliveryPipe<'cfg, 'db, 'api>>> {
        self.inner.iter()
    }
}
