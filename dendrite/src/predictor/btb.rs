//! Branch target buffer (BTB) implementations.

use crate::predictor::*;
use crate::branch::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimpleBTBEntry {
    /// Cached target address for this branch
    pub tgt: usize,
    /// The type of branch
    pub kind: BranchKind,
}
impl SimpleBTBEntry {
    pub fn new(tgt: usize, kind: BranchKind) -> Self { 
        Self { tgt, kind }
    }

    pub fn target(&self) -> usize { self.tgt }
    pub fn kind(&self) -> BranchKind { self.kind }
}

pub struct SimpleBTB {
    size: usize,
    data: Vec<Option<SimpleBTBEntry>>,
}
impl SimpleBTB {
    pub fn new(size: usize) -> Self {
        assert!(size.is_power_of_two());
        Self { 
            size,
            data: vec![None; size],
        }
    }
}

//impl PredictorTable for SimpleBTB {
//    /// The type of input to the table used to form an index.
//    type Input<'a> = usize;
//
//    type Index = usize;
//
//    /// The type of entry in the table.
//    type Entry = SimpleBTBEntry;
//
//    /// Returns the number of entries in the table.
//    fn size(&self) -> usize { self.size }
//
//    /// Given some input, return the corresponding index into the table. 
//    //fn get_index(&self, input: Self::Input) -> usize {
//    //    (self.index_fn)(input) & self.index_mask()
//    //}
//
//    /// Returns a reference to an entry in the table.
//    fn get_entry(&self, idx: Self::Index) -> &Self::Entry {
//        &self.data[idx]
//    }
//
//    /// Returns a mutable reference to an entry in the table.
//    fn get_entry_mut(&mut self, idx: Self::Index) -> &mut Self::Entry {
//        &mut self.data[idx]
//    }
//}
