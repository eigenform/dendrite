
use crate::direction::*;
use crate::history::*;

pub trait IndexedByPc {
    fn index_from_pc(&self, pc: usize) -> usize;
}
pub trait TaggedByPc {
    fn tag_from_pc(&self, pc: usize) -> usize;
}

pub trait PredictFromPc {
    fn predict(&self, pc: usize) -> Outcome;
}
