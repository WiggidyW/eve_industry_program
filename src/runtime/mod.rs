use std::rc::Rc;

mod delivery_route;

use delivery_route::*;

mod location;
use location::*;

mod delivery_pipe;
use delivery_pipe::*;

mod production_line;
use production_line::*;

mod market_orders;
use market_orders::*;

pub struct RuntimeData<'cfg, 'db, 'api> {
    pub locations: Vec<Rc<Location<'cfg, 'db, 'api>>>,
}
