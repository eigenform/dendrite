
use crate::Outcome;
use crate::history::*;
use crate::predictor::*;
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct TAGEBaseConfig {
    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,

    /// Number of entries
    pub size: usize,

    /// Strategy for indexing into the table.
    pub index_strat: IndexStrategy<TAGEBaseComponent>,
}
impl TAGEBaseConfig {
    pub fn storage_bits(&self) -> usize { 
        self.ctr.storage_bits() * self.size
    }

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
    type Input<'a> = TAGEInputs<'a>;
    type Index = usize;
    type Entry = SaturatingCounter;

    fn size(&self) -> usize { self.cfg.size }

    fn get_index(&self, input: TAGEInputs) -> usize { 
        let res = match self.cfg.index_strat {
            IndexStrategy::FromPc(func) => { 
                (func)(self, input.pc)
            },
            IndexStrategy::FromPhr(func) => { 
                (func)(self, input.pc, input.phr)
            },
        };
        res & self.index_mask()
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


/// An entry in some [TAGEComponent]. 
#[derive(Clone, Debug)]
pub struct TAGEEntry {
    pub ctr: SaturatingCounter,
    pub useful_bits: usize,
    pub useful: u8,
    pub tag: Option<usize>,
    pub updates: usize,
}
impl TAGEEntry { 
    pub fn new(ctr: SaturatingCounter, useful_bits: usize) -> Self { 
        Self { ctr, useful_bits, useful: 0, tag: None, updates: 0 }
    }

    /// Get the current predicted outcome.
    pub fn predict(&self) -> Outcome {
        self.ctr.predict()
    }

    /// Update the saturating counter associated with this entry.
    pub fn update(&mut self, outcome: Outcome) {
        let prediction = self.predict();
        if prediction == outcome { 
            self.useful += 1;
        } else { 
            self.useful -= 1;
        }
        self.ctr.update(outcome);
    }

    /// Returns true if the provided tag matches this entry. 
    pub fn tag_matches(&self, tag: usize) -> bool { 
        if let Some(val) = self.tag { val == tag } else { false }
    }

    /// Increment the 'useful' counter.
    pub fn increment_useful(&mut self) {
        self.useful = (self.useful + 1).clamp(0, (1 << self.useful_bits) - 1);
    }

    /// Decrement the 'useful' counter.
    pub fn decrement_useful(&mut self) {
        self.useful = (self.useful - 1).clamp(0, (1 << self.useful_bits) - 1);
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

    pub useful_bits: usize,

    /// Strategy for indexing into the table
    pub index_strat: IndexStrategy<TAGEComponent>,

    /// Strategy for creating tags
    pub tag_strat: TagStrategy<TAGEComponent>,

    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,
}
impl TAGEComponentConfig {
    pub fn storage_bits(&self) -> usize { 
        let entry_size = (
            self.ctr.storage_bits() +
            self.useful_bits +
            self.tag_bits
        );
        entry_size * self.size
    }

    pub fn build(self) -> TAGEComponent {
        assert!(self.size.is_power_of_two());
        let csr = FoldedHistoryRegister::new(
            self.size.ilog2() as usize,
            self.ghr_range.clone()
        );
        let entry = TAGEEntry::new(self.ctr.build(), self.useful_bits);
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
impl TAGEComponent {
    pub fn reset_useful_bits(&mut self) {
        for entry in self.data.iter_mut() {
            entry.useful = 0;
        }
    }
}

impl PredictorTable for TAGEComponent {
    type Input<'a> = TAGEInputs<'a>;
    type Index = usize;
    type Entry = TAGEEntry;

    fn size(&self) -> usize { self.cfg.size }

    fn get_index(&self, input: TAGEInputs) -> usize { 
        let res = match self.cfg.index_strat {
            IndexStrategy::FromPc(func) => { 
                (func)(self, input.pc)
            },
            IndexStrategy::FromPhr(func) => { 
                (func)(self, input.pc, input.phr)
            },
        };
        res & self.index_mask()
    }


    //fn get_index(&self, pc: usize) -> usize { 
    //    let ghist_bits = self.csr.output_usize(); 
    //    let pc_bits = (self.cfg.pc_sel_fn)(pc);
    //    let index = ghist_bits ^ pc_bits;
    //    index & self.index_mask()
    //}

    fn get_entry(&self, idx: usize) -> &TAGEEntry { 
        let index = idx & self.index_mask();
        &self.data[index]
    }
    fn get_entry_mut(&mut self, idx: usize) -> &mut TAGEEntry { 
        let index = idx & self.index_mask();
        &mut self.data[index]
    }

}

impl <'a> TaggedPredictorTable<'a> for TAGEComponent {
    fn get_tag(&self, input: TAGEInputs) -> usize { 
        match self.cfg.tag_strat {
            TagStrategy::FromPc(func) => (func)(self, input.pc)
        }
        //let pc_bits = (self.cfg.pc_sel_fn)(pc); 
        //let ghist0_bits = self.csr.output_usize();
        //let ghist1_bits = self.csr.output_usize() << 1;
        //pc_bits ^ ghist0_bits ^ ghist1_bits
    }
}

