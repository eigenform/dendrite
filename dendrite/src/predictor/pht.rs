//! Implementations of a pattern history table (PHT). 

use crate::Outcome;
use crate::predictor::*;
use crate::predictor::counter::*;

/// A table of [SaturatingCounter] indexed by the program counter. 
pub struct SimplePHT { 
    /// Saturating counter configuration
    cfg: SaturatingCounterConfig,

    /// Table of counters
    data: Vec<SaturatingCounter>,

    /// Number of entries
    size: usize,

    /// Index function
    index_fn: fn(&Self, pc: usize) -> usize,
}
impl SimplePHT {
    pub fn new(size: usize, 
        index_fn: fn(&Self, pc: usize) -> usize,
        cfg: SaturatingCounterConfig
    ) -> Self
    { 
        let data = vec![cfg.build(); size];
        Self { 
            cfg,
            data,
            size,
            index_fn,
        }
    }
}

impl PredictorTable for SimplePHT {
    type Input<'a> = usize;
    type Index = usize;
    type Entry = SaturatingCounter;

    fn size(&self) -> usize { self.size }

    fn get_index(&self, pc: usize) -> usize { 
        (self.index_fn)(self, pc) & self.index_mask()
    }

    fn get_entry(&self, idx: usize) -> &SaturatingCounter { 
        let index = idx & self.index_mask();
        &self.data[index]
    }

    fn get_entry_mut(&mut self, idx: usize) -> &mut SaturatingCounter { 
        let index = idx & self.index_mask();
        &mut self.data[index]
    }
}


