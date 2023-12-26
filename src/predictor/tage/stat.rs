
use std::collections::*;

/// Container for [TAGEPredictor] runtime stats.
#[derive(Debug)]
pub struct TAGEStats {
    /// Successful allocations
    pub alcs: usize,

    /// Failed allocations
    pub failed_alcs: usize,

    /// Misses in the base component
    pub base_miss: usize,

    /// Misses in the tagged components
    pub comp_miss: Vec<usize>,

    /// Number of 'useful' counter resets
    pub resets: usize,

    /// Number of updates
    pub clk: usize,
}
impl TAGEStats {
    pub fn new(num_comp: usize) -> Self { 
        Self {
            alcs: 0,
            failed_alcs: 0,
            base_miss: 0,
            comp_miss: vec![0; num_comp],
            resets: 0,
            clk: 0,
        }
    }
}

/// Container for [TAGEEntry] runtime stats.
#[derive(Clone, Debug)]
pub struct TAGEEntryStats {
    /// Number of updates
    pub updates: usize,

    /// Number of invalidations
    pub invalidations: usize,

    /// Set of unique program counter values for branches that were 
    /// allocated/tracked in this entry
    pub branches: BTreeSet<usize>,

    /// Number of [TAGEPredictor] updates since the last invalidation.
    pub clk: usize,
}
impl TAGEEntryStats {
    pub fn new() -> Self { 
        Self {
            updates: 0,
            invalidations: 0,
            branches: BTreeSet::new(),
            clk: 0,
        }
    }

    pub fn was_unused(&self) -> bool {
        self.branches.is_empty()
    }

    pub fn num_aliasing_branches(&self) -> usize { 
        self.branches.len()
    }

}


