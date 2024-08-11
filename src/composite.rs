use futures::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use std::collections::{HashMap, HashSet};

use crate::{
    config::{self, Item},
    industry_db,
};

pub async fn get_db_lines(
    cfg_locations: impl Iterator<Item = &config::Location>,
    cfg_skills: &HashMap<u32, u8>,
    cfg_max_time: std::time::Duration,
    db: &impl industry_db::IndustryDatabase,
) -> Result<HashMap<u32, industry_db::Line>, crate::Error> {
    let mut db_line_futs = FuturesUnordered::new();
    for location in cfg_locations {
        let location_production = match &location.production {
            Some(location_production) => location_production,
            None => continue,
        };
        for production_line in location_production.production_lines.iter() {
            let fut = db
                .compute_line(
                    location.system_id,
                    location_production.structure_type_id,
                    location_production.rigs,
                    location_production.tax,
                    cfg_skills,
                    production_line.kind,
                    production_line.transput,
                    cfg_max_time,
                    production_line.decryptor,
                )
                .map_ok(move |line| (production_line.id, line));
            db_line_futs.push(fut);
        }
    }
    let mut db_lines = HashMap::new();
    while let Some(result) = db_line_futs.try_next().await? {
        let (id, line) = result;
        db_lines.insert(id, line);
    }
    Ok(db_lines)
}

pub async fn get_db_volumes_and_names(
    cfg_locations: impl Iterator<Item = &config::Location>,
    db_lines: impl Iterator<Item = &industry_db::Line>,
    db: &impl industry_db::IndustryDatabase,
) -> Result<(HashMap<Item, f64>, HashMap<Item, String>), crate::Error> {
    let mut seen_items = HashSet::new();

    let mut db_volume_futs_1 = FuturesUnordered::new();
    let mut db_name_futs_1 = FuturesUnordered::new();
    for line in db_lines {
        for &(item, _) in line.minerals.iter() {
            if seen_items.insert(item) {
                db_volume_futs_1.push(
                    db.get_volume(item).map_ok(move |volume| (item, volume)),
                );
                db_name_futs_1
                    .push(db.get_name(item).map_ok(move |name| (item, name)));
            }
        }
    }

    let mut db_volume_futs_2 = FuturesUnordered::new();
    let mut db_name_futs_2 = FuturesUnordered::new();
    let mut db_volume_futs_3 = FuturesUnordered::new();
    let mut db_name_futs_3 = FuturesUnordered::new();
    for location in cfg_locations {
        let location_production = match &location.production {
            Some(location_production) => location_production,
            None => continue,
        };
        for production_line in location_production.production_lines.iter() {
            if seen_items.insert(production_line.transput.product) {
                db_volume_futs_2.push(
                    db.get_volume(production_line.transput.product).map_ok(
                        move |volume| {
                            (production_line.transput.product, volume)
                        },
                    ),
                );
                db_name_futs_2.push(
                    db.get_name(production_line.transput.product).map_ok(
                        move |name| (production_line.transput.product, name),
                    ),
                );
            }
            if seen_items.insert(production_line.transput.blueprint) {
                db_volume_futs_3.push(
                    db.get_volume(production_line.transput.blueprint).map_ok(
                        move |volume| {
                            (production_line.transput.blueprint, volume)
                        },
                    ),
                );
                db_name_futs_3.push(
                    db.get_name(production_line.transput.blueprint).map_ok(
                        move |name| (production_line.transput.blueprint, name),
                    ),
                );
            }
        }
    }

    let mut db_volumes = HashMap::new();
    while let Some(result) = db_volume_futs_1.try_next().await? {
        let (item, volume) = result;
        if let Some(volume) = volume {
            db_volumes.insert(item, volume);
        }
    }
    while let Some(result) = db_volume_futs_2.try_next().await? {
        let (item, volume) = result;
        if let Some(volume) = volume {
            db_volumes.insert(item, volume);
        }
    }
    while let Some(result) = db_volume_futs_3.try_next().await? {
        let (item, volume) = result;
        if let Some(volume) = volume {
            db_volumes.insert(item, volume);
        }
    }

    let mut db_names = HashMap::new();
    while let Some(result) = db_name_futs_1.try_next().await? {
        let (item, name) = result;
        db_names.insert(item, name);
    }
    while let Some(result) = db_name_futs_2.try_next().await? {
        let (item, name) = result;
        db_names.insert(item, name);
    }
    while let Some(result) = db_name_futs_3.try_next().await? {
        let (item, name) = result;
        db_names.insert(item, name);
    }

    Ok((db_volumes, db_names))
}
