//! For classifying conditional branch behavior.

use crate::analysis::*;

/// Classifications for conditional branch behavior. 
///
///
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BranchClass {
    Unknown,

    /// A branch whose outcome is always the same
    Static(Outcome),

    /// A branch with a totally uniform repeating pattern of outcomes
    /// that divides the entire local history. 

    UniformPattern(Vec<RunPair<Outcome>>),

    UniformPatternPrefixed(usize, Vec<Outcome>),

}


