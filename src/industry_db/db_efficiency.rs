#[derive(Debug, Clone, Copy)]
pub enum Security {
    High,
    Low,
    Zero, // Null OR Wormhole OR Pochven
}

impl From<f64> for Security {
    fn from(value: f64) -> Self {
        if value >= 0.45 {
            Security::High
        } else if value > 0.0 {
            Security::Low
        } else {
            Security::Zero
        }
    }
}

#[derive(Default)]
pub enum SkillLevel {
    #[default]
    One,
    Two,
    Three,
    Four,
    Five,
}

impl From<u8> for SkillLevel {
    fn from(value: u8) -> Self {
        match value {
            1 => SkillLevel::One,
            2 => SkillLevel::Two,
            3 => SkillLevel::Three,
            4 => SkillLevel::Four,
            5 => SkillLevel::Five,
            _ => unreachable!(),
        }
    }
}

impl From<u32> for SkillLevel {
    fn from(value: u32) -> Self {
        match value {
            1 => SkillLevel::One,
            2 => SkillLevel::Two,
            3 => SkillLevel::Three,
            4 => SkillLevel::Four,
            5 => SkillLevel::Five,
            _ => unreachable!(),
        }
    }
}

impl Into<f64> for SkillLevel {
    fn into(self) -> f64 {
        match self {
            SkillLevel::One => 1.0,
            SkillLevel::Two => 2.0,
            SkillLevel::Three => 3.0,
            SkillLevel::Four => 4.0,
            SkillLevel::Five => 5.0,
        }
    }
}

pub struct Efficiency {
    material_efficiency: f64,
    time_efficiency: f64,
    cost_efficiency: f64,
    zero_sec_multiplier: f64,
    low_sec_multiplier: f64,
    high_sec_multiplier: f64,
    probability_multiplier: f64,
}

impl Efficiency {
    pub fn new(
        time_efficiency: f64,
        material_efficiency: f64,
        cost_efficiency: f64,
        probability_multiplier: f64,
        high_sec_multiplier: f64,
        low_sec_multiplier: f64,
        zero_sec_multiplier: f64,
    ) -> Efficiency {
        Efficiency {
            material_efficiency: material_efficiency,
            time_efficiency: time_efficiency,
            cost_efficiency: cost_efficiency,
            zero_sec_multiplier: zero_sec_multiplier,
            low_sec_multiplier: low_sec_multiplier,
            high_sec_multiplier: high_sec_multiplier,
            probability_multiplier: probability_multiplier,
        }
    }

    fn security_multipler(&self, security: Security) -> f64 {
        match security {
            Security::High => self.high_sec_multiplier,
            Security::Low => self.low_sec_multiplier,
            Security::Zero => self.zero_sec_multiplier,
        }
    }

    fn mult(
        &self,
        e: &mut f64,
        level_mult: f64,
        security_mult: f64,
        mult: f64,
    ) {
        *e *= 1.0 - mult * level_mult * security_mult
    }

    fn add_probability_inner(&self, probability: &mut f64, level_mult: f64) {
        if self.probability_multiplier > 0.0 {
            *probability += self.probability_multiplier * level_mult;
        }
    }

    pub fn add_probability(&self, probability: &mut f64, level: SkillLevel) {
        self.add_probability_inner(probability, level.into());
    }

    pub fn add_efficiencies(
        &self,
        me: &mut f64,
        te: &mut f64,
        ce: &mut f64,
        probability: &mut f64,
        level: SkillLevel,
        security: Security,
    ) {
        let level_mult = level.into();
        let security_mult = self.security_multipler(security);
        if self.material_efficiency > 0.0 {
            self.mult(me, level_mult, security_mult, self.material_efficiency);
        }
        if self.time_efficiency > 0.0 {
            self.mult(te, level_mult, security_mult, self.time_efficiency);
        }
        if self.cost_efficiency > 0.0 {
            self.mult(ce, level_mult, security_mult, self.cost_efficiency);
        }
        self.add_probability_inner(probability, level_mult);
    }
}
