
use std::ops::{ Add, Sub, RangeInclusive };

/// A branch outcome. 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { N, T }
impl std::ops::Not for Outcome { 
    type Output = Self;
    fn not(self) -> Self { 
        match self { 
            Self::N => Self::T,
            Self::T => Self::N,
        }
    }
}
impl From<bool> for Outcome {
    fn from(x: bool) -> Self { 
        match x { 
            true => Self::T,
            false => Self::N 
        }
    }
}
impl Into<bool> for Outcome {
    fn into(self) -> bool { 
        match self { 
            Self::T => true,
            Self::N => false,
        }
    }
}



/// An 'n'-bit saturating counter used to follow the behavior of a branch. 
#[derive(Clone, Copy, Debug)]
pub struct SaturatingCounter {
    max_t_state: u8,
    max_n_state: u8,
    default: Outcome,
    state: Outcome,
    ctr: u8,
}
impl SaturatingCounter {
    pub fn new(max_t_state: u8, max_n_state: u8, default: Outcome)
        -> Self 
    {
        Self { 
            max_t_state,
            max_n_state,
            default,
            state: default,
            ctr: 0,
        }
    }

    pub fn strengthen(&mut self) {
        let lim = match self.state { 
            Outcome::T => self.max_t_state,
            Outcome::N => self.max_n_state,
        };
        self.ctr = (self.ctr + 1).clamp(0, lim);
    }

    pub fn weaken(&mut self) {
        if let Some(next) = self.ctr.checked_sub(1) { 
            self.ctr = next;
        } else { 
            self.state = !self.state;
        }
    }

    pub fn set_direction(&mut self, outcome: Outcome) {
        self.state = outcome;
    }

    /// Reset the counter.
    pub fn reset(&mut self) { 
        self.state = self.default; 
        self.ctr = 0;
    }

    /// Return the current predicted direction.
    pub fn predict(&self) -> Outcome { self.state }

    /// Update the state of the counter. 
    pub fn update(&mut self, outcome: Outcome) {
        let prediction = self.predict();
        if outcome != prediction {
            self.weaken();
        } else { 
            self.strengthen();
        }
    }
}




