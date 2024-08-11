use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub enum IndustrySlot {
    Manufacturing,
    Reaction,
    Science,
}

impl IndustrySlot {
    pub fn from_activity_id(activity_id: i32) -> Option<IndustrySlot> {
        Some(match activity_id {
            // 0 => None, // means None in game
            1 => IndustrySlot::Manufacturing,
            3 => IndustrySlot::Science, // Research TE
            4 => IndustrySlot::Science, // Research ME
            5 => IndustrySlot::Science, // Copying
            7 => IndustrySlot::Science, // Reverse Engineering (deprecated in game)
            8 => IndustrySlot::Science, // Invention
            9 => IndustrySlot::Reaction, // https://github.com/esi/esi-issues/issues/997
            11 => IndustrySlot::Reaction,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndustrySlots {
    pub manufacturing: usize,
    pub reaction: usize,
    pub science: usize,
}

impl IndustrySlots {
    pub fn use_slot(&mut self, slot: IndustrySlot) -> bool {
        match slot {
            IndustrySlot::Manufacturing => match self.manufacturing {
                0 => false,
                _ => {
                    self.manufacturing -= 1;
                    true
                }
            },
            IndustrySlot::Reaction => match self.reaction {
                0 => false,
                _ => {
                    self.reaction -= 1;
                    true
                }
            },
            IndustrySlot::Science => match self.science {
                0 => false,
                _ => {
                    self.science -= 1;
                    true
                }
            },
        }
    }

    pub fn can_use_slot(&self, slot: IndustrySlot) -> bool {
        match slot {
            IndustrySlot::Manufacturing => self.manufacturing > 0,
            IndustrySlot::Reaction => self.reaction > 0,
            IndustrySlot::Science => self.science > 0,
        }
    }

    pub fn can_use_slots(&self, slots: &Self) -> bool {
        self.manufacturing >= slots.manufacturing
            && self.reaction >= slots.reaction
            && self.science >= slots.science
    }

    pub fn use_slot_unwrap(&mut self, slot: IndustrySlot) {
        if !self.use_slot(slot) {
            panic!("No available slot for {:?}", slot);
        }
    }

    pub fn available(&self, slot: IndustrySlot) -> bool {
        match slot {
            IndustrySlot::Manufacturing => match self.manufacturing {
                0 => false,
                _ => true,
            },
            IndustrySlot::Reaction => match self.reaction {
                0 => false,
                _ => true,
            },
            IndustrySlot::Science => match self.science {
                0 => false,
                _ => true,
            },
        }
    }

    pub fn from_slot(slot: IndustrySlot) -> Self {
        match slot {
            IndustrySlot::Manufacturing => IndustrySlots {
                manufacturing: 1,
                reaction: 0,
                science: 0,
            },
            IndustrySlot::Reaction => IndustrySlots {
                manufacturing: 0,
                reaction: 1,
                science: 0,
            },
            IndustrySlot::Science => IndustrySlots {
                manufacturing: 0,
                reaction: 0,
                science: 1,
            },
        }
    }

    pub fn add(&mut self, slots: IndustrySlots) {
        self.manufacturing += slots.manufacturing;
        self.reaction += slots.reaction;
        self.science += slots.science;
    }
}
