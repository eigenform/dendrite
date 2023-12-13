
use crate::direction::*;
use crate::history::*;
use crate::predictor::*;
use std::ops::RangeInclusive;

/// A TAGE "base predictor" component. 
pub struct TAGEBaseComponent {
    /// A table of saturating counters
    data: Vec<SaturatingCounter>,
    /// The number of entries in the table
    size: usize,
    /// Function used to form an index from a program counter value
    index_fn: fn(usize) -> usize,
}
impl TAGEBaseComponent {
    pub fn new(
        ctr: SaturatingCounter, 
        size: usize, 
        index_fn: fn(usize) -> usize
    ) -> Self 
    { 
        Self { 
            data: vec![ctr; size],
            size, 
            index_fn 
        }
    }
    pub fn update(&mut self, pc: usize, outcome: Outcome) {
        let idx = self.index_from_pc(pc);
        self.data[idx].update(outcome);
    }
}
impl IndexedByPc for TAGEBaseComponent {
    fn index_from_pc(&self, pc: usize) -> usize { 
        (self.index_fn)(pc)
    }
}
impl PredictFromPc for TAGEBaseComponent {
    fn predict(&self, pc: usize) -> Outcome { 
        let idx = self.index_from_pc(pc);
        self.data[idx].predict()
    }
}


/// An entry in some [TAGEComponent]. 
#[derive(Clone, Debug)]
pub struct TAGEEntry {
    ctr: SaturatingCounter,
    useful: u8,
    tag: Option<usize>,
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

    pub fn matches(&self, tag: usize) -> bool { 
        if let Some(val) = self.tag { 
            val == tag
        } else { 
            false
        }
    }

    /// Invalidate this entry.
    pub fn invalidate(&mut self) {
        self.ctr.reset();
        self.useful = 0;
        self.tag = None;
    }
}


/// A tagged component in the TAGE predictor. 
#[derive(Clone, Debug)]
pub struct TAGEComponent {
    /// Table of entries
    pub data: Vec<TAGEEntry>,
    /// Number of entries
    pub size: usize,
    /// Relevant slice in global history
    pub ghr_range: RangeInclusive<usize>,
    /// Number of tag bits
    pub tag_bits: usize,
    /// Function selecting relevant program counter bits
    pub pc_sel_fn: fn(usize) -> usize,
    /// Folded global history
    pub csr: FoldedHistoryRegister,
}
impl TAGEComponent { 
    pub fn new(
        entry: TAGEEntry,
        size: usize, 
        ghr_range: RangeInclusive<usize>,
        tag_bits: usize,
        pc_sel_fn: fn(usize) -> usize,
    ) -> Self
    {
        Self { 
            data: vec![entry; size],
            size,
            ghr_range: ghr_range.clone(), 
            tag_bits,
            pc_sel_fn,
            csr: FoldedHistoryRegister::new(
                size.ilog2() as usize, 
                ghr_range.clone(),
            ),
        }
    }
}
impl TAGEComponent { 

    pub fn lookup(&self, pc: usize) -> Option<&TAGEEntry> {
        let index = self.index_from_pc(pc);
        let tag   = self.tag_from_pc(pc);
        let entry = &self.data[index];
        if entry.matches(tag) {
            Some(entry)
        } else { 
            None 
        }
    }
    pub fn lookup_mut(&mut self, pc: usize) -> Option<&mut TAGEEntry> {
        let index = self.index_from_pc(pc);
        let tag   = self.tag_from_pc(pc);
        let entry = &mut self.data[index];
        if entry.matches(tag) {
            Some(entry)
        } else { 
            None 
        }
    }


    pub fn try_predict(&self, pc: usize) -> Option<Outcome> { 
        if let Some(entry) = self.lookup(pc) {
            Some(entry.predict())
        } else { 
            None
        }
    }

    pub fn allocate(&mut self, pc: usize) {
        let index = self.index_from_pc(pc);
        let tag   = self.tag_from_pc(pc);
        let entry = &mut self.data[index];
        entry.invalidate();
        entry.ctr.inc();
        entry.tag = Some(tag);
    }

    pub fn update_history(&mut self, ghr: &GlobalHistoryRegister) {
        self.csr.update(ghr);
    }

}
impl IndexedByPc for TAGEComponent {
    fn index_from_pc(&self, pc: usize) -> usize { 
        self.csr.output_usize() ^ (self.pc_sel_fn)(pc)
    }
}
impl TaggedByPc for TAGEComponent {
    fn tag_from_pc(&self, pc: usize) -> usize { 
        0
    }
}


