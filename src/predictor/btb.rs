
use crate::predictor::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimpleBTBEntry {
    tgt: usize,
}
impl SimpleBTBEntry {
    pub fn new(tgt: usize) -> Self { 
        Self {
            tgt,
        }
    }

    pub fn target(&self) -> usize { self.tgt }
}

pub struct SimpleBTB {
    size: usize,
    data: Vec<Option<SimpleBTBEntry>>,
    index_fn: PcToIndexFn,
}
impl SimpleBTB {
    pub fn new(size: usize, index_fn: PcToIndexFn) -> Self {
        assert!(size.is_power_of_two());
        Self { 
            size,
            data: vec![None; size],
            index_fn,
        }
    }
}

