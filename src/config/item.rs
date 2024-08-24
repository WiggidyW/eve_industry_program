use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Deserialize)]
pub struct Item {
    pub type_id: u32,
    #[serde(default)]
    pub runs: i16,
    #[serde(default)]
    pub me: i8,
    #[serde(default)]
    pub te: i8,
}

impl Item {
    pub fn new(type_id: u32) -> Item {
        Item {
            type_id: type_id,
            runs: 0,
            me: 0,
            te: 0,
        }
    }

    pub fn new_blueprint(type_id: u32, runs: i16, me: i8, te: i8) -> Item {
        Item {
            type_id: type_id,
            runs: runs,
            me: me,
            te: te,
        }
    }

    pub const fn null() -> Item {
        Item {
            type_id: 0,
            runs: 0,
            me: 0,
            te: 0,
        }
    }

    pub fn into_non_blueprint(self) -> Item {
        Item {
            type_id: self.type_id,
            runs: 0,
            me: 0,
            te: 0,
        }
    }

    pub fn is_null(&self) -> bool {
        self.type_id == 0
    }

    pub fn is_blueprint(&self) -> bool {
        self.runs != 0
    }

    pub fn is_bpc(&self) -> bool {
        self.runs > 0
    }

    pub fn is_bpo(&self) -> bool {
        self.runs == -1
    }

    pub fn is_marketable(&self) -> bool {
        self.runs == 0 || self.runs == -1 && self.me == 0 && self.te == 0
    }
}

impl From<u32> for Item {
    fn from(value: u32) -> Self {
        Item {
            type_id: value,
            runs: 0,
            me: 0,
            te: 0,
        }
    }
}
