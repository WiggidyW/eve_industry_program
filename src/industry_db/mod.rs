use crate::config::{self, IndustrySlot, Item, ManufacturingKind, Transput};

use std::collections::HashMap;
use std::iter;
use std::time::Duration;

mod db;
mod db_efficiency;
mod db_response;
mod decryptors;
mod line;
mod sqlite_db;
mod static_data;
mod volume;

pub use line::Line;
pub use volume::Volume;

use db::{DatabaseParamsInclude, InnerDatabase};
use db_efficiency::*;
use db_response::DatabaseResponse;
use decryptors::*;
use static_data::*;

pub trait IndustryDatabase: Send + Sync {
    async fn compute_line(
        &self,
        // location config
        system_id: u32,
        structure_id: u32,
        rigs: [Option<u32>; 3],
        tax: config::ManufacturingValue,
        // config
        skills: &HashMap<u32, u8>,
        // production line
        kind: config::ManufacturingKind,
        transput: config::Transput,
        max_duration: Duration,
        decryptor: Option<u32>,
    ) -> Result<Line, crate::Error>;
    async fn get_volume(
        &self,
        item: Item,
    ) -> Result<Option<Volume>, crate::Error>;
    async fn get_name(&self, item: Item) -> Result<String, crate::Error>;
}

pub async fn new_industry_database(
) -> Result<impl IndustryDatabase, crate::Error> {
    sqlite_db::SqliteDb::connect()
        .await
        .map_err(|e| crate::Error::IndustryDbError(e.into()))
}
