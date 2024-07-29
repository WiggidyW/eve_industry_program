use super::*;
use crate::config;
use std::{cell::RefCell, rc::Rc};

pub struct DeliveryRoute<'cfg, 'db, 'api> {
    pub inner: &'cfg config::DeliveryRoute,
    pub src: Rc<Location<'cfg, 'db, 'api>>,
    pub dst: Rc<Location<'cfg, 'db, 'api>>,
    // pub pipes: RefCell<Vec<Rc<DeliveryPipe<'cfg, 'db, 'api>>>>,
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
            // pipes: RefCell::new(Vec::new()),
        }
    }

    pub fn delivery_rate(&self) -> &config::DeliveryRate {
        &self.inner.rate
    }
}
