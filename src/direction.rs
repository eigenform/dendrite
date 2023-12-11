
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


/// An n-bit saturating counter used to follow the behavior of a branch. 
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
    pub fn inc(&mut self) { self.state = self.clamp(self.state.add(1)); }
    pub fn dec(&mut self) { self.state = self.clamp(self.state.sub(1)); }

    pub fn reset(&mut self) {
        self.state = self.init;
    }

    pub fn output(&self) -> Outcome {
        if self.state == 0 {
            self.default
        } else { 
            Outcome::from(self.state.is_positive())
        }
    }
}




