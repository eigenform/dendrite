
use crate::Outcome;
use crate::history::*;
use crate::predictor::*;
use std::ops::RangeInclusive;

#[derive(Clone, Copy, Debug)]
pub struct TAGEBaseConfig {
    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,

    /// Number of entries
    pub size: usize,

    /// Index function
    pub index_fn: PcToIndexFn,
}
impl TAGEBaseConfig {
    pub fn build(self) -> TAGEBaseComponent {
        assert!(self.size.is_power_of_two());
        TAGEBaseComponent {
            data: vec![self.ctr.build(); self.size],
            cfg: self,
        }
    }
}


/// A base component in the TAGE predictor. 
#[derive(Clone, Debug)]
pub struct TAGEBaseComponent {
    cfg: TAGEBaseConfig,
    /// A table of saturating counters
    data: Vec<SaturatingCounter>,
}
impl TAGEBaseComponent {
    pub fn index_mask(&self) -> usize { 
        self.cfg.size - 1
    }
}
impl PredictorTable for TAGEBaseComponent {
    type Input = usize;
    type Entry = SaturatingCounter;

    fn size(&self) -> usize { self.cfg.size }
    fn get_index(&self, pc: usize) -> usize { 
        (self.cfg.index_fn)(pc) & self.index_mask()
    }
    fn get_entry(&self, pc: usize) -> &SaturatingCounter { 
        let index = self.get_index(pc);
        &self.data[index]
    }
    fn get_entry_mut(&mut self, pc: usize) -> &mut SaturatingCounter { 
        let index = self.get_index(pc);
        &mut self.data[index]
    }

}


/// An entry in some [TAGEComponent]. 
#[derive(Clone, Debug)]
pub struct TAGEEntry {
    pub ctr: SaturatingCounter,
    pub useful: u8,
    pub tag: Option<usize>,
}
impl TAGEEntry { 
    pub fn new(ctr: SaturatingCounter) -> Self { 
        Self { ctr, useful: 0, tag: None, }
    }

    pub fn predict(&self) -> Outcome {
        self.ctr.predict()
    }

    pub fn update(&mut self, outcome: Outcome) {
        let prediction = self.predict();
        if prediction == outcome { 
            self.useful += 1;
        } else { 
            self.useful -= 1;
        }
        self.ctr.update(outcome);
    }

    pub fn tag_match(&self, tag: usize) -> bool { 
        if let Some(val) = self.tag { val == tag } else { false }
    }

    pub fn increment_useful(&mut self) {
        self.useful = (self.useful + 1) & 0b11;
    }

    /// Invalidate this entry.
    pub fn invalidate(&mut self) {
        self.ctr.reset();
        self.useful = 0;
        self.tag = None;
    }
}

#[derive(Clone, Debug)]
pub struct TAGEComponentConfig {
    /// Number of entries
    pub size: usize,

    /// Relevant slice in global history
    pub ghr_range: RangeInclusive<usize>,

    /// Number of tag bits
    pub tag_bits: usize,

    /// Function selecting relevant program counter bits
    pub pc_sel_fn: PcBitSelectFn,

    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,
}
impl TAGEComponentConfig {
    pub fn build(self) -> TAGEComponent {
        assert!(self.size.is_power_of_two());
        let csr = FoldedHistoryRegister::new(
            self.size.ilog2() as usize,
            self.ghr_range.clone()
        );
        let entry = TAGEEntry::new(self.ctr.build());
        let data = vec![entry; self.size];

        TAGEComponent {
            cfg: self,
            data,
            csr,
        }
    }
}

/// A tagged component in the TAGE predictor. 
#[derive(Clone, Debug)]
pub struct TAGEComponent {
    pub cfg: TAGEComponentConfig,
    /// Table of entries
    pub data: Vec<TAGEEntry>,
    /// Folded global history
    pub csr: FoldedHistoryRegister,
}
impl PredictorTable for TAGEComponent {
    type Input = usize;
    type Entry = TAGEEntry;

    fn size(&self) -> usize { self.cfg.size }
    fn get_index(&self, pc: usize) -> usize { 
        let ghist_bits = self.csr.output_usize(); 
        let pc_bits = (self.cfg.pc_sel_fn)(pc);
        let index = ghist_bits ^ pc_bits;
        index & self.index_mask()
    }
    fn get_entry(&self, pc: usize) -> &TAGEEntry { 
        let index = self.get_index(pc);
        &self.data[index]
    }
    fn get_entry_mut(&mut self, pc: usize) -> &mut TAGEEntry { 
        let index = self.get_index(pc);
        &mut self.data[index]
    }

}

impl TaggedPredictorTable for TAGEComponent {
    fn get_tag(&self, pc: usize) -> usize { 
        let pc_bits = (self.cfg.pc_sel_fn)(pc); 
        let ghist0_bits = self.csr.output_usize();
        let ghist1_bits = self.csr.output_usize() << 1;
        pc_bits ^ ghist0_bits ^ ghist1_bits
    }
}

