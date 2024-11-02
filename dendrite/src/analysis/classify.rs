//! For classifying conditional branch behavior.

use crate::analysis::*;

/// Classifications for conditional branch behavior. 
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BranchClass {
    Unknown,

    /// A branch where the outcome is always the same.
    Static(Outcome),

    /// A branch where the outcome changes only once.
    SinglePair(RunPair<Outcome>),

    /// A branch where the "head" of each pair is always the same length,
    /// and only the "tail" length is variable
    StaticHead(Vec<RunPair<Outcome>>),

    /// A branch where the "tail" of each pair is always the same length,
    /// and only the "head" length is variable
    StaticTail(Vec<RunPair<Outcome>>),

    /// A branch with a totally uniform repeating pattern of outcomes
    /// that divides the entire local history. 
    UniformPattern(Vec<RunPair<Outcome>>),

    UniformPatternPrefixed(usize, Vec<Outcome>),
}


