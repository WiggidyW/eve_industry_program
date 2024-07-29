use crate::config::ManufacturingKind;
use super::SqliteKind;

pub const fn into_database_kind(kind: ManufacturingKind) -> SqliteKind {
    match kind {
        ManufacturingKind::Manufacturing => 1,
        ManufacturingKind::Invention => 2,
        ManufacturingKind::Copy => 3,
        ManufacturingKind::Reaction => 4,
    }
}

pub const fn from_database_kind(kind: SqliteKind) -> ManufacturingKind {
    match kind {
        1 => ManufacturingKind::Manufacturing,
        2 => ManufacturingKind::Invention,
        3 => ManufacturingKind::Copy,
        4 => ManufacturingKind::Reaction,
        _ => unreachable!(),
    }
}
