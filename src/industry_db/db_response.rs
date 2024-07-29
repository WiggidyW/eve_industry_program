use super::*;

pub struct DatabaseResponse {
    pub product: Item,
    pub probability: f64,
    pub portion: Quantity, // per 1 run
    pub duration: Duration,
    pub minerals: Vec<(Item, Quantity)>,
    pub installation_minerals: Vec<(Item, Quantity)>,
    pub efficiencies: HashMap<TypeId, Efficiency>,
    pub security: f64,
}

impl DatabaseResponse {
    pub fn add_efficiencies(
        &self,
        type_id: &TypeId,
        me: &mut f64,
        te: &mut f64,
        ce: &mut f64,
        probability: &mut f64,
        level: SkillLevel,
        security: Security,
    ) {
        if let Some(eff) = self.efficiencies.get(type_id) {
            eff.add_efficiencies(me, te, ce, probability, level, security);
        }
    }

    pub fn add_probability(
        &self,
        type_id: &TypeId,
        probability: &mut f64,
        level: SkillLevel,
    ) {
        if let Some(eff) = self.efficiencies.get(type_id) {
            eff.add_probability(probability, level);
        }
    }
}