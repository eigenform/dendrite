
use crate::direction::*;
use crate::history::*;

pub trait PredictorTable { 
    /// The type of input to the table used to form an index.
    type Input;

    /// The type of entry in the table.
    type Entry;

    /// Returns the number of entries in the table.
    fn size(&self) -> usize;

    /// The hash function used to form an index into the table.
    fn get_index(&self, input: Self::Input) -> usize;

    fn get_entry(&self, input: Self::Input) -> &Self::Entry;
    fn get_entry_mut(&mut self, input: Self::Input) -> &mut Self::Entry;

    /// Returns a mask corresponding to the number of entries in the table.
    fn index_mask(&self) -> usize { 
        assert!(self.size().is_power_of_two());
        self.size() - 1
    }
}

pub trait TaggedPredictorTable: PredictorTable {
    fn get_tag(&self, input: Self::Input) -> usize;
}

pub trait IndexedByPc {
    fn index_from_pc(&self, pc: usize) -> usize;
}
pub trait TaggedByPc {
    fn tag_from_pc(&self, pc: usize) -> usize;
}

pub trait PredictFromPc {
    fn predict(&self, pc: usize) -> Outcome;
}
