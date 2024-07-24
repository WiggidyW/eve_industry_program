#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(u32);

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

pub enum BuildKind {
    Manufacturing,
    Copying,
    Invention,
    // ReverseEngineering,
    Reaction,
}
