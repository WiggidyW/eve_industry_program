use super::*;

pub struct Line {
    pub installation_minerals: Vec<(Item, Quantity)>, // minerals to be used for computing installation cost for N runs
    pub minerals: Vec<(Item, Quantity)>, // minerals needed for N runs
    pub portion: Quantity,               // number produced from N runs
    pub duration: Duration,              // time needed for N runs
    pub runs: Quantity,                  // number of runs
    pub cost_multiplier: f64, // job cost (0.0 - 1.0) * job kind multiplier (0.02 or 1.0) + that * tax
}

impl Line {
    pub const fn database_params_include() -> DatabaseParamsInclude {
        DatabaseParamsInclude {
            minerals: true,
            installation_minerals: true,
            efficiencies: true,
            security: true,
        }
    }

    pub fn from_rep(
        db_rep: DatabaseResponse,
        structure_id: TypeId,
        rigs: [Option<TypeId>; 3],
        tax: config::ManufacturingValue,
        skills: &HashMap<u32, u8>,
        kind: config::ManufacturingKind,
        transput: config::Transput,
        max_duration: Duration,
        decryptor: Option<TypeId>,
    ) -> Result<Line, crate::Error> {
        let num_runs = match kind {
            ManufacturingKind::Copy => Some(transput.product.runs.into()),
            ManufacturingKind::Manufacturing => {
                match transput.blueprint.is_bpc() {
                    true => Some(transput.blueprint.runs.into()),
                    false => None,
                }
            }
            _ => None,
        };

        let mut material_efficiency = 1.0;
        let mut time_efficiency = 1.0;
        let mut cost_efficiency = 1.0;
        let mut probability = db_rep.probability;
        let security = db_rep.security.into();

        for (type_id, slvl) in iter::once(structure_id)
            .map(|id| (id, SkillLevel::One))
            .chain(rigs.into_iter().flatten().map(|id| (id, SkillLevel::One)))
            .chain(skills.into_iter().map(|(id, slvl)| (*id, (*slvl).into())))
        {
            db_rep.add_efficiencies(
                &type_id,
                &mut material_efficiency,
                &mut time_efficiency,
                &mut cost_efficiency,
                &mut probability,
                slvl,
                security,
            );
        }
        add_blueprint_efficiencies(
            &mut material_efficiency,
            &mut time_efficiency,
            &transput.blueprint,
            kind,
        );

        let run_once_duration = db_rep.duration.mul_f64(time_efficiency);
        let mut max_runs_f64 = (max_duration.as_secs_f64()
            / run_once_duration.as_secs_f64())
        .floor();
        let mut max_runs_qnt: i64 = max_runs_f64 as Quantity;
        if let Some(num_runs) = num_runs {
            if max_runs_qnt > num_runs {
                max_runs_qnt = num_runs;
                max_runs_f64 = num_runs as f64;
            } else {
                return Err(crate::Error::Unimplemented);
            }
        }

        let mut line = Line {
            installation_minerals: db_rep.installation_minerals,
            minerals: db_rep.minerals,
            portion: db_rep.portion * max_runs_qnt,
            duration: run_once_duration.mul_f64(max_runs_f64),
            runs: max_runs_qnt,
            cost_multiplier: kind_multiplier(kind) // 1.0 or 0.02
                * cost_efficiency // 0.0 - 1.0
                * (1.0 + tax.kind_value(kind)), // 1.0 - 2.0
        };

        for (_, quantity) in &mut line.minerals {
            if material_efficiency == 1.0 || quantity == &1 {
                *quantity *= max_runs_qnt; // 1-ofs are always 1 per run regardless of material efficiency
            } else {
                *quantity =
                    (*quantity as f64 * max_runs_f64 * material_efficiency)
                        .ceil() as Quantity;
            }
        }
        for (_, quantity) in &mut line.installation_minerals {
            *quantity *= max_runs_qnt;
        }
        if transput.blueprint.is_bpc() {
            line.minerals.push((transput.blueprint, 1));
        }

        match kind {
            ManufacturingKind::Invention => line.set_invention(
                db_rep.product,
                transput.product,
                probability,
                max_runs_qnt,
                decryptor,
            )?,
            ManufacturingKind::Copy => line.set_copy(),
            _ => (),
        };

        Ok(line)
    }

    fn set_invention(
        &mut self,
        db_product: Item,
        line_product: Item,
        mut probability: f64,
        max_runs: Quantity,
        decryptor: Option<TypeId>,
    ) -> Result<(), crate::Error> {
        if let Some(decryptor) = decryptor {
            if db_product == line_product {
                // configured decryptor is invalid
                return Err(crate::Error::Unimplemented);
            } else {
                match find_matching_decryptor(
                    db_product,
                    line_product,
                    decryptor,
                ) {
                    Some((decryptor, pmult)) => {
                        self.minerals.push((decryptor, max_runs));
                        probability *= pmult;
                    }
                    // configured decryptor is invalid
                    None => return Err(crate::Error::Unimplemented),
                }
            }
        }
        self.portion = (self.portion as f64 * probability).floor() as Quantity;
        Ok(())
    }

    fn set_copy(&mut self) {
        self.portion = 1;
        self.runs = 1;
    }
}

fn add_blueprint_efficiencies(
    me: &mut f64,
    te: &mut f64,
    blueprint: &Item,
    kind: config::ManufacturingKind,
) {
    if kind == ManufacturingKind::Manufacturing {
        *me *= 1.0 - blueprint.me as f64 / 100.0;
        *te *= 1.0 - blueprint.te as f64 / 100.0;
    }
}
