//! Types for implementing a table of predictors.

use crate::Outcome;
use crate::history::*;

///// A function used to create an index from a program counter value. 
/////
///// Ideally this is some kind of hash function.
//pub type PcIndexFn<T> = fn(&T, pc: usize) -> usize;

///// A function used to create an index from (a) a program counter value, and;
///// (b) a reference to some [HistoryRegister] used for path history. 
/////
///// Ideally this is some kind of hash function.
//pub type PhrIndexFn<T> = 
//    fn(&T, pc: usize, phr: &HistoryRegister) -> usize;

///// A function used to create an index from some input.
//pub type GenericIndexFn<T, I, O> = fn(&T, I) -> O;
//
///// A function used to create a tag from some input.
//pub type GenericTagFn<T, I, O>   = fn(&T, I) -> O;


/// A user-provided strategy for indexing into a [PredictorTable]. 
#[derive(Clone, Copy, Debug)]
pub enum IndexStrategy<T> {
    FromPc(fn(&T, pc: usize) -> usize),
}


/// A user-provided strategy for generating a tag associated with some 
/// entry in a [PredictorTable].
#[derive(Clone, Copy, Debug)]
pub enum TagStrategy<T> {
    FromPc(fn(&T, pc: usize) -> usize),
}

/// Interface to a table of predictors. 
pub trait PredictorTable: Sized { 
    /// The type of input to the table used to form an index.
    type Input<'a>;

    /// The type of an index into the table.
    type Index;

    /// The type of entry in the table.
    type Entry;

    /// The signature of functions used to generate an index.
    type IndexFn<'a> = fn(&Self, Self::Input<'a>) -> Self::Index;

    /// Returns the number of entries in the table.
    fn size(&self) -> usize;

    /// Given some input, generate an index into the table. 
    fn get_index(&self, input: Self::Input<'_>) -> Self::Index;

    /// Returns a reference to an entry in the table.
    fn get_entry(&self, idx: Self::Index) -> &Self::Entry;

    /// Returns a mutable reference to an entry in the table.
    fn get_entry_mut(&mut self, idx: Self::Index) -> &mut Self::Entry;

    /// Returns a bitmask corresponding to the number of entries in the table.
    fn index_mask(&self) -> usize { 
        assert!(self.size().is_power_of_two());
        self.size() - 1
    }
}

/// Interface to a *tagged* table of predictors. 
pub trait TaggedPredictorTable<'a>: PredictorTable {
    /// The type of a tag associated with each entry in the table. 
    type Tag<'b>;

    /// The signature of functions used to generate a tag. 
    type TagFn<'b> = fn(&Self, Self::Input<'a>) -> Self::Tag<'b>;

    /// Given some input, generate a tag used to match an entry in the table.
    fn get_tag(&self, input: Self::Input<'a>) -> usize;
}


