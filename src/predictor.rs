
pub mod simple;
pub mod tage;
pub mod counter; 
pub mod perceptron;
pub mod btb; 

pub use counter::*;
pub use perceptron::*;
pub use tage::*;
pub use btb::*;

use crate::history::*;
use crate::Outcome;


/// Some hash function used to create an index from a program counter value. 
pub type PcIndexFn<T> = fn(&T, pc: usize) -> usize;

/// Some hash function used to create an index from a program counter value 
/// and a reference to some [HistoryRegister] used for path history. 
pub type PhrIndexFn<T> = 
    fn(&T, pc: usize, phr: &HistoryRegister) -> usize;

/// A user-provided strategy for indexing into some object implementing 
/// [PredictorTable].
///
#[derive(Clone, Copy, Debug)]
pub enum IndexStrategy<T> {
    FromPc(PcIndexFn<T>),
    FromPhr(PhrIndexFn<T>),
}


/// A user-provided strategy for generating a tag associated with some 
/// entry in an object implementing [PredictorTable].
#[derive(Clone, Copy, Debug)]
pub enum TagStrategy<T> {
    FromPc(PcIndexFn<T>),
}

/// Interface to a "trivial" predictor that simply guesses an outcome. 
pub trait SimplePredictor {
    fn name(&self) -> &'static str;
    fn predict(&self) -> Outcome;
}


/// Interface to a table of predictors. 
pub trait PredictorTable { 
    /// The type of input to the table used to form an index.
    type Input<'a>;

    /// The type of an index into the table.
    type Index;

    /// The type of entry in the table.
    type Entry;

    /// Returns the number of entries in the table.
    fn size(&self) -> usize;

    /// Given some input, return the corresponding index into the table. 
    fn get_index(&self, input: Self::Input<'_>) -> Self::Index;

    /// Returns a reference to an entry in the table.
    fn get_entry(&self, idx: Self::Index) -> &Self::Entry;

    /// Returns a mutable reference to an entry in the table.
    fn get_entry_mut(&mut self, idx: Self::Index) -> &mut Self::Entry;

    /// Returns a mask corresponding to the number of entries in the table.
    fn index_mask(&self) -> usize { 
        assert!(self.size().is_power_of_two());
        self.size() - 1
    }
}

/// Interface to a *tagged* table of predictors. 
pub trait TaggedPredictorTable<'a>: PredictorTable {
    fn get_tag(&self, input: Self::Input<'a>) -> usize;
}

