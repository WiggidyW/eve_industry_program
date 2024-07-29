use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Item {
    pub type_id: TypeId,
    pub runs: i16,
    pub me: i8,
    pub te: i8,
}

impl Item {
    pub fn new(type_id: TypeId) -> Item {
        Item {
            type_id: type_id,
            runs: 0,
            me: 0,
            te: 0,
        }
    }

    pub fn new_blueprint(type_id: TypeId, runs: i16, me: i8, te: i8) -> Item {
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

impl From<TypeId> for Item {
    fn from(value: TypeId) -> Self {
        Item {
            type_id: value,
            runs: 0,
            me: 0,
            te: 0,
        }
    }
}
