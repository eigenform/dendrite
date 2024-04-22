//! Implementation of a saturating counter.

use crate::Outcome;
use crate::predictor::StatefulPredictor;

/// Configuration for building a [`SaturatingCounter`].
#[derive(Clone, Copy, Debug)]
pub struct SaturatingCounterConfig {
    pub max_t_state: u8,
    pub max_n_state: u8,
    pub default_state: Outcome,
}
impl SaturatingCounterConfig {
    pub fn storage_bits(&self) -> usize {
        (self.max_t_state.ilog2() + self.max_n_state.ilog2() + 1)
            as usize
    }
    pub fn build(self) -> SaturatingCounter {
        SaturatingCounter {
            cfg: self,
            state: self.default_state,
            ctr: 0,
        }
    }
}

/// An N-bit saturating counter used to follow the behavior of a branch. 
#[derive(Clone, Copy, Debug)]
pub struct SaturatingCounter {
    cfg: SaturatingCounterConfig,
    state: Outcome,
    ctr: u8,
}
impl SaturatingCounter {
    pub fn strengthen(&mut self) {
        let lim = match self.state { 
            Outcome::T => self.cfg.max_t_state,
            Outcome::N => self.cfg.max_n_state,
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

    /// Set the strength of the current prediction.
    pub fn set_strength(&mut self, val: u8) {
        let lim = match self.state { 
            Outcome::T => self.cfg.max_t_state,
            Outcome::N => self.cfg.max_n_state,
        };
        self.ctr = val.clamp(0, lim);
    }

    /// Set the current predicted direction.
    pub fn set_direction(&mut self, outcome: Outcome) {
        self.state = outcome;
    }
}

impl StatefulPredictor for SaturatingCounter { 
    fn name(&self) -> &'static str { "SaturatingCounter" }
    fn predict(&self) -> Outcome { self.state }
    fn reset(&mut self) { 
        self.state = self.cfg.default_state; 
        self.ctr = 0;
    }
    fn update(&mut self, outcome: Outcome) {
        let prediction = self.predict();
        if outcome != prediction {
            self.weaken();
        } else { 
            self.strengthen();
        }
    }
}


