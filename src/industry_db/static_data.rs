use crate::config::{Item, ManufacturingKind};

pub const DEFAULT_INVENTION_ME: i8 = 2;
pub const DEFAULT_INVENTION_TE: i8 = 4;
pub const DECRYPTORS: [(Item, f64); 7] = [
    (
        Item {
            type_id: 34203,
            runs: 9,
            me: -2,
            te: 2,
        },
        0.6,
    ),
    (
        Item {
            type_id: 34208,
            runs: 7,
            me: 2,
            te: 0,
        },
        0.9,
    ),
    // (Item { // Support for Symmetry Decryptor is not enabled at this time
    //     type_id: 34206,
    //     runs: 2,
    //     me: 1,
    //     te: 8,
    // },
    // 1.0),
    (
        Item {
            type_id: 34205,
            runs: 0,
            me: 3,
            te: 6,
        },
        1.1,
    ),
    (
        Item {
            type_id: 34201,
            runs: 1,
            me: 2,
            te: 10,
        },
        1.2,
    ),
    (
        Item {
            type_id: 34204,
            runs: 3,
            me: 1,
            te: -2,
        },
        1.5,
    ),
    (
        Item {
            type_id: 34202,
            runs: 4,
            me: -1,
            te: 4,
        },
        1.8,
    ),
    (
        Item {
            type_id: 34207,
            runs: 2,
            me: 1,
            te: -2,
        },
        1.9,
    ),
];

pub const fn kind_multiplier(kind: ManufacturingKind) -> f64 {
    match kind {
        ManufacturingKind::Manufacturing => 1.00,
        ManufacturingKind::Invention => 0.02,
        ManufacturingKind::Copy => 0.02,
        ManufacturingKind::Reaction => 1.00,
    }
}
