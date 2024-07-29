use super::*;

use futures::stream::TryStreamExt;
use sqlx::{self, SqlitePool};

mod static_data;
mod typedef;

use static_data::*;
use typedef::*;

pub struct SqliteDb {
    inner: SqlitePool,
}

impl InnerDatabase for SqliteDb {
    type Error = sqlx::Error;
    async fn get(
        &self,
        product_id: TypeId,
        blueprint: Item,
        kind: ManufacturingKind,
        system_id: SystemId,
        include: DatabaseParamsInclude,
    ) -> Result<DatabaseResponse, Self::Error> {
        let mut conn = self.inner.acquire().await?;
        let (blueprint, identifiers) =
            select_blueprint(&mut conn, product_id, blueprint, kind).await?;
        let minerals = match include.minerals {
            true => select_minerals(&mut conn, identifiers.minerals).await?,
            false => Vec::new(),
        };
        let installation_minerals = match include.installation_minerals {
            true => {
                if identifiers.minerals == identifiers.installation_minerals
                    && include.minerals
                {
                    minerals.clone()
                } else {
                    select_minerals(
                        &mut conn,
                        identifiers.installation_minerals,
                    )
                    .await?
                }
            }
            false => Vec::new(),
        };
        let efficiencies = match include.efficiencies {
            true => {
                select_rigs_skills_structures(
                    &mut conn,
                    kind,
                    identifiers.rigs_skills_structures,
                )
                .await?
            }
            false => HashMap::new(),
        };
        let security = match include.security {
            true => select_security(&mut conn, system_id).await?,
            false => 0.0,
        };
        Ok(DatabaseResponse {
            product: blueprint.product,
            probability: blueprint.probability,
            portion: blueprint.portion,
            duration: blueprint.duration,
            minerals: minerals,
            installation_minerals: installation_minerals,
            efficiencies: efficiencies,
            security: security,
        })
    }
    async fn get_volume(&self, item: TypeId) -> Result<Volume, Self::Error> {
        unimplemented!()
    }
}

impl SqliteDb {
    pub async fn connect() -> sqlx::Result<SqliteDb> {
        Ok(SqliteDb {
            inner: SqlitePool::connect("db.sqlite").await?,
        })
    }
    // fn query_blueprint(
    //     &self,

    // )
}

struct SqliteIdentifiers {
    rigs_skills_structures: SqliteID,
    installation_minerals: SqliteID,
    minerals: SqliteID,
}

struct Blueprint {
    product: Item,
    portion: Quantity,
    probability: f64,
    duration: Duration,
}

struct DbBlueprint {
    portion: i64,
    probability: f64,
    duration: i64,
    rigs_skills_structures: SqliteID,
    installation_minerals: SqliteID,
    minerals: SqliteID,
}

async fn select_blueprint(
    conn: &mut SqlitePoolConnection,
    product_id: TypeId,
    blueprint: Item,
    kind: ManufacturingKind,
) -> sqlx::Result<(Blueprint, SqliteIdentifiers)> {
    let db_kind = into_database_kind(kind);
    sqlx::query_file_as!(
        DbBlueprint,
        "sqlite_build_data/select_blueprint.sql",
        product_id,
        blueprint.type_id,
        db_kind,
    )
    .fetch_one(conn)
    .await
    .map(|b| {
        (
            Blueprint {
                product: match kind {
                    ManufacturingKind::Copy => Item::new_blueprint(
                        product_id,
                        b.portion as i16,
                        blueprint.me,
                        blueprint.te,
                    ),
                    ManufacturingKind::Invention => Item::new_blueprint(
                        product_id,
                        b.portion as i16,
                        DEFAULT_INVENTION_ME,
                        DEFAULT_INVENTION_TE,
                    ),
                    _manufacturing_or_reaction => Item::new(product_id),
                },
                portion: match kind.is_science() {
                    true => 1,
                    false => b.portion,
                },
                probability: b.probability,
                duration: Duration::from_secs(b.duration as u64),
            },
            SqliteIdentifiers {
                rigs_skills_structures: b.rigs_skills_structures,
                installation_minerals: b.installation_minerals,
                minerals: b.minerals,
            },
        )
    })
}

struct DbMineral {
    type_id: i64,
    quantity: i64,
}

async fn select_minerals(
    conn: &mut SqlitePoolConnection,
    id: SqliteID,
) -> sqlx::Result<Vec<(Item, Quantity)>> {
    sqlx::query_file_as!(
        DbMineral,
        "sqlite_build_data/select_minerals.sql",
        id,
    )
        .fetch(conn)
        .map_ok(|m| (Item::new(m.type_id as TypeId), m.quantity))
        .try_collect()
        .await
}

struct DbEfficiency {
    type_id: i64,
    time_efficiency: f64,
    material_efficiency: f64,
    cost_efficiency: f64,
    probability_multiplier: f64,
    high_sec_multiplier: f64,
    low_sec_multiplier: f64,
    zero_sec_multiplier: f64,
}

async fn select_rigs_skills_structures(
    conn: &mut SqlitePoolConnection,
    kind: ManufacturingKind,
    id: SqliteID,
) -> sqlx::Result<HashMap<TypeId, Efficiency>> {
    let kind = into_database_kind(kind);
    sqlx::query_file_as!(
        DbEfficiency,
        "sqlite_build_data/select_rigs_skills_structures.sql",
        kind,
        id,
    )
    .fetch(conn)
    .map_ok(|e| {
        (
            e.type_id as TypeId,
            Efficiency::new(
                e.time_efficiency,
                e.material_efficiency,
                e.cost_efficiency,
                e.zero_sec_multiplier,
                e.low_sec_multiplier,
                e.high_sec_multiplier,
                e.probability_multiplier,
            ),
        )
    })
    .try_collect()
    .await
}

struct DbSecurity {
    security: f64,
}

async fn select_security(
    conn: &mut SqlitePoolConnection,
    system_id: SystemId,
) -> sqlx::Result<f64> {
    sqlx::query_file_as!(
        DbSecurity,
        "sqlite_build_data/select_security.sql",
        system_id,
    )
    .fetch_one(conn)
    .await
    .map(|s| s.security)
}

struct DbVolume {
    volume: f64,
}

async fn select_volume(
    conn: &mut SqlitePoolConnection,
    type_id: TypeId,
) -> sqlx::Result<Volume> {
    sqlx::query_file_as!(
        DbVolume,
        "sqlite_build_data/select_volume.sql",
        type_id,
    )
    .fetch_one(conn)
    .await
    .map(|v| v.volume)
}
