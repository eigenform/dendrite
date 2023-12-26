
use crate::Outcome;
use crate::history::*;
use crate::predictor::*;
use std::ops::RangeInclusive;


/// A base component in the TAGE predictor. 
#[derive(Clone, Debug)]
pub struct TAGEBaseComponent {
    pub cfg: TAGEBaseConfig,

    /// A table of saturating counters
    pub data: Vec<SaturatingCounter>,
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
    /// Container for debugging and analysis data
    pub stat: TAGEEntryStats,

    /// State machine tracking a branch outcome
    pub ctr: SaturatingCounter,

    /// The number of bits in the 'useful' counter
    pub useful_bits: usize,

    /// The 'useful' counter, used to determine when the entry is 
    /// eligible to be invalidated and replaced
    pub useful: u8,

    /// Tag associated with this entry
    pub tag: Option<usize>,
}
impl TAGEEntry { 
    pub fn new(ctr: SaturatingCounter, useful_bits: usize) -> Self { 
        Self { 
            ctr, 
            useful_bits, 
            useful: 0, 
            tag: None, 
            stat: TAGEEntryStats::new(),

        }
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

        self.stat.updates += 1;
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
    pub fn invalidate(&mut self, clk: usize) {
        self.ctr.reset();
        self.useful = 0;
        self.tag = None;
        self.stat.invalidations += 1;

        //self.stat.generations.push(clk);
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
    pub fn num_useful_entries(&self) -> usize { 
        self.data.iter().filter(|e| e.useful != 0).count()
    }

    /// Calculate what percentage of entries have been allocated. 
    pub fn utilization(&self) -> f64 { 
        let unused_entries = self.data.iter().filter(|e| e.stat.was_unused())
            .count() as f64;
        (1.0 - (unused_entries / self.data.len() as f64)) * 100.0
    }

    /// Reset the 'useful' counter for all entries in this component.
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

