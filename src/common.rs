#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Item {
    Item(TypeId),
    Blueprint(Blueprint),
}

impl Item {
    pub fn type_id(&self) -> TypeId {
        match self {
            Item::Item(type_id) => *type_id,
            Item::Blueprint(blueprint) => blueprint.type_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Blueprint {
    pub type_id: TypeId,
    pub me: MaterialEfficiency,
    pub te: TimeEfficiency,
    pub runs: u8, // 0 for BPO
}

pub enum BlueprintKind {
    BPO,
    BPC,
}

impl Blueprint {
    pub fn kind(&self) -> BlueprintKind {
        if self.runs == 0 {
            BlueprintKind::BPO
        } else {
            BlueprintKind::BPC
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocationId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SystemId(u32);

pub enum SlotKind {
    Manufacturing,
    Reaction,
    Science,
}

pub struct SlotCount {
    pub manufacture: u32,
    pub reaction: u32,
    pub science: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JobKind {
    Manufacturing,
    Copying,
    Invention,
    // ReverseEngineering,
    Reaction,
}

impl JobKind {
    pub fn installation_cost_multiplier(&self) -> f64 {
        match self {
            JobKind::Manufacturing => 1.0,
            JobKind::Copying => 0.02,
            JobKind::Invention => 0.02,
            JobKind::Reaction => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MaterialEfficiency {
    ME00,
    ME01,
    ME02,
    ME03,
    ME04,
    ME05,
    ME06,
    ME07,
    ME08,
    ME09,
    ME10,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimeEfficiency {
    TE00,
    TE01,
    TE02,
    TE03,
    TE04,
    TE05,
    TE06,
    TE07,
    TE08,
    TE09,
    TE10,
}
