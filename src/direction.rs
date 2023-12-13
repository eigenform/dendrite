
use std::ops::{ Add, Sub, RangeInclusive };

/// A branch outcome. 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { N, T }
impl From<bool> for Outcome {
    fn from(x: bool) -> Self { 
        match x { 
            true => Self::T,
            false => Self::N 
        }
    }
}


/// An 'n'-bit saturating counter used to follow the behavior of a branch. 
#[derive(Clone, Copy, Debug)]
pub struct SaturatingCounter {
    lo: i8, 
    hi: i8,
    default: Outcome,
    init: i8,
    state: i8,
}
impl SaturatingCounter {
    pub fn new(range: RangeInclusive<i8>, init: i8, default: Outcome) -> Self { 
        assert!(range.start().is_negative() && range.end().is_positive());
        Self { 
            lo: *range.start(),
            hi: *range.end(),
            init,
            state: init,
            default,
        }
    }

    fn clamp(&self, x: i8) -> i8 { x.clamp(self.lo, self.hi) }

    /// Increment the counter.
    pub fn inc(&mut self) { self.state = self.clamp(self.state.add(1)); }

    /// Decrement the counter.
    pub fn dec(&mut self) { self.state = self.clamp(self.state.sub(1)); }

    /// Reset the counter.
    pub fn reset(&mut self) { self.state = self.init; }

    /// Return the current predicted direction.
    pub fn predict(&self) -> Outcome {
        if self.state == 0 {
            self.default
        } else { 
            Outcome::from(self.state.is_positive())
        }
    }

    /// Update the state of the counter. 
    pub fn update(&mut self, outcome: Outcome) {
        let current_prediction = self.predict();
        let misprediction = outcome != current_prediction;
        match outcome { 
            Outcome::N => {
                if misprediction { self.inc() } else { self.dec() }
            },
            Outcome::T => {
                if misprediction { self.dec() } else { self.inc() }
            },
        }
    }
}




