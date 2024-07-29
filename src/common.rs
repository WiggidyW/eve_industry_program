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

    pub fn me(&self, item: Item, job_kind: JobKind) -> f64 {
        self.me.blueprint_me(item, job_kind)
    }

    pub fn te(&self, item: Item, job_kind: JobKind) -> f64 {
        self.te.blueprint_te(item, job_kind)
    }

    pub fn ce(&self, item: Item, job_kind: JobKind) -> f64 {
        1.0
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SlotCount {
    pub manufacture: u32,
    pub reaction: u32,
    pub science: u32,
}

impl SlotCount {
    pub fn new() -> SlotCount {
        SlotCount {
            manufacture: 0,
            reaction: 0,
            science: 0,
        }
    }

    pub fn can_fit(&self, other: &SlotCount) -> bool {
        self.manufacture >= other.manufacture
            && self.reaction >= other.reaction
            && self.science >= other.science
    }

    pub fn reserve(&mut self, job: JobKind) {
        match job {
            JobKind::Manufacturing => self.manufacture -= 1,
            JobKind::Copying => self.science -= 1,
            JobKind::Invention => self.science -= 1,
            JobKind::Reaction => self.reaction -= 1,
        }
    }

    pub fn add(&mut self, slot: JobKind) {
        match slot {
            JobKind::Manufacturing => self.manufacture += 1,
            JobKind::Copying => self.science += 1,
            JobKind::Invention => self.science += 1,
            JobKind::Reaction => self.reaction += 1,
        }
    }
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

impl MaterialEfficiency {
    pub fn blueprint_me(&self, item: Item, job_kind: JobKind) -> f64 {
        match (job_kind, self) {
            (JobKind::Manufacturing, MaterialEfficiency::ME00) => 1.0,
            (JobKind::Manufacturing, MaterialEfficiency::ME01) => 0.99,
            (JobKind::Manufacturing, MaterialEfficiency::ME02) => 0.98,
            (JobKind::Manufacturing, MaterialEfficiency::ME03) => 0.97,
            (JobKind::Manufacturing, MaterialEfficiency::ME04) => 0.96,
            (JobKind::Manufacturing, MaterialEfficiency::ME05) => 0.95,
            (JobKind::Manufacturing, MaterialEfficiency::ME06) => 0.94,
            (JobKind::Manufacturing, MaterialEfficiency::ME07) => 0.93,
            (JobKind::Manufacturing, MaterialEfficiency::ME08) => 0.92,
            (JobKind::Manufacturing, MaterialEfficiency::ME09) => 0.91,
            (JobKind::Manufacturing, MaterialEfficiency::ME10) => 0.90,
            _ => 1.0,
        }
    }
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

impl TimeEfficiency {
    pub fn blueprint_te(&self, item: Item, job_kind: JobKind) -> f64 {
        match (job_kind, self) {
            (JobKind::Manufacturing, TimeEfficiency::TE00) => 1.0,
            (JobKind::Manufacturing, TimeEfficiency::TE01) => 0.98,
            (JobKind::Manufacturing, TimeEfficiency::TE02) => 0.96,
            (JobKind::Manufacturing, TimeEfficiency::TE03) => 0.94,
            (JobKind::Manufacturing, TimeEfficiency::TE04) => 0.92,
            (JobKind::Manufacturing, TimeEfficiency::TE05) => 0.90,
            (JobKind::Manufacturing, TimeEfficiency::TE06) => 0.88,
            (JobKind::Manufacturing, TimeEfficiency::TE07) => 0.86,
            (JobKind::Manufacturing, TimeEfficiency::TE08) => 0.84,
            (JobKind::Manufacturing, TimeEfficiency::TE09) => 0.82,
            (JobKind::Manufacturing, TimeEfficiency::TE10) => 0.80,
            _ => 1.0,
        }
    }
}
