// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub trait Strength: Clone + Copy + PartialEq + PartialOrd + Default {
    const DECAY_RATE: f64;
    const GROWTH_RATE: f64 = 1.0 - Self::DECAY_RATE;

    fn new(incr_value: bool) -> Self;
    fn value(&self) -> f64;
    fn increase(&mut self);
    fn decrease(&mut self);

    fn incremented(&self) -> Self {
        let mut strength = *self;
        strength.increase();
        strength
    }

    fn decremented(&self) -> Self {
        let mut strength = *self;
        strength.decrease();
        strength
    }
}

// SIMPLE STRENGTH

#[derive(Debug, Default, Clone, Copy, PartialOrd, PartialEq)]
pub struct SimpleStrength(f64);

impl Strength for SimpleStrength {
    const DECAY_RATE: f64 = 0.95;

    fn new(incr_value: bool) -> Self {
        let mut ss = Self::default();
        if incr_value {
            ss.increase()
        }
        ss
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn increase(&mut self) {
        self.0 += (1.0 - self.0) * Self::GROWTH_RATE;
    }

    fn decrease(&mut self) {
        self.0 *= Self::DECAY_RATE;
    }
}
