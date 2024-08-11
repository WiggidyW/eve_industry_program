use super::*;

pub struct DbLineTransformed<'db> {
    inner: &'db industry_db::Line,
    extra_duration: Duration,
    runs_multiplier: i64,
}

impl<'db> DbLineTransformed<'db> {
    pub fn new(
        inner: &'db industry_db::Line,
        max_time: Duration,
        daily_flex_time: Duration,
    ) -> Self {
        let (extra_duration, runs_multiplier) = if (inner.duration * 2)
            > max_time
        {
            (Duration::new(0, 0), 1)
        } else {
            let flexed_time_per_sequence = inner.duration + daily_flex_time;
            // enforce flexed_time_per_sequence to be a multiple of 24 hours
            let final_time_per_sequence = flexed_time_per_sequence
                + Duration::from_secs(24 * 60 * 60)
                - Duration::new(
                    flexed_time_per_sequence.as_secs() % (24 * 60 * 60),
                    flexed_time_per_sequence.subsec_nanos(),
                );
            let multiplier =
                (max_time.as_secs() / final_time_per_sequence.as_secs()) as i64;
            let extra_duration = final_time_per_sequence - inner.duration;
            (extra_duration, multiplier)
        };
        Self {
            inner,
            extra_duration,
            runs_multiplier,
        }
    }

    pub fn installation_minerals(
        &self,
    ) -> impl Iterator<Item = (Item, i64)> + '_ {
        self.inner
            .installation_minerals
            .iter()
            .map(move |(item, quantity)| {
                (*item, *quantity * self.runs_multiplier)
            })
    }

    pub fn minerals(&self) -> impl Iterator<Item = (Item, i64)> + '_ {
        self.inner.minerals.iter().map(move |(item, quantity)| {
            (*item, *quantity * self.runs_multiplier)
        })
    }

    pub fn cost_multiplier(&self) -> f64 {
        self.inner.cost_multiplier
    }

    pub fn runs(&self) -> i64 {
        self.inner.runs * self.runs_multiplier
    }

    pub fn portion(&self) -> i64 {
        self.inner.portion * self.runs_multiplier
    }
}
