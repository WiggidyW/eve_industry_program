use std::ops::{Add, AddAssign, Sub};

#[derive(Debug, Clone)]
pub struct Profit {
    pub cost: f64,
    pub revenue: f64,
}

impl Profit {
    pub fn new(cost: f64, revenue: f64) -> Self {
        Self { cost, revenue }
    }

    pub fn margin(&self) -> f64 {
        self.revenue / self.cost
    }

    pub fn margin_percent(&self) -> f64 {
        self.margin() * 100.0
    }

    pub fn profit(&self) -> f64 {
        self.revenue - self.cost
    }
}

impl AddAssign for Profit {
    fn add_assign(&mut self, other: Self) {
        self.cost += other.cost;
        self.revenue += other.revenue;
    }
}

impl Add for Profit {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cost: self.cost + other.cost,
            revenue: self.revenue + other.revenue,
        }
    }
}
