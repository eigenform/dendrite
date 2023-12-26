
use crate::Outcome;

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

/// An 'n'-bit saturating counter used to follow the behavior of a branch. 
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

    pub fn set_direction(&mut self, outcome: Outcome) {
        self.state = outcome;
    }

    /// Reset the counter.
    pub fn reset(&mut self) { 
        self.state = self.cfg.default_state; 
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



