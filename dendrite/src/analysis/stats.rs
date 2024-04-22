//! Helpers for collecting statistics.

use std::collections::*;
use bitvec::prelude::*;
use itertools::*;

use crate::branch::*;
use crate::analysis::*;

/// Container for recording simple statistics while iterating over a trace.
pub struct BranchStats {
    /// Per-branch data (indexed by program counter value)
    pub data: BTreeMap<usize, BranchData>,

    /// Number of correct predictions
    pub global_hits: usize,

    /// Number of times any branch instruction was executed
    pub global_brns: usize,
}
impl BranchStats {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            global_hits: 0,
            global_brns: 0,
        }
    }

    /// Return the global hit rate.
    pub fn hit_rate(&self) -> f64 {
        self.global_hits as f64 / self.global_brns as f64
    }

    /// Return the global hit count.
    pub fn global_hits(&self) -> usize { self.global_hits }

    /// Return the global miss count.
    pub fn global_miss(&self) -> usize { self.global_brns - self.global_hits }

    /// Return the total branch count.
    pub fn global_brns(&self) -> usize { self.global_brns }

    /// Update global statistics.
    pub fn update_global(&mut self, record: &BranchRecord, outcome: Outcome) {
        let hit = outcome == record.outcome;
        self.global_brns += 1;
        if hit { self.global_hits += 1; }
    }

    /// Update per-branch statistics.
    pub fn update_per_branch(&mut self,
        record: &BranchRecord, outcome: Outcome)
    {
        let hit = outcome == record.outcome;
        let data = self.get_mut(record.pc);
        data.occ += 1;
        data.outcomes.push(outcome);
        if hit { data.hits += 1; }
    }

    /// Returns a reference to data collected for a particular branch.
    pub fn get(&self, pc: usize) -> Option<&BranchData> {
        self.data.get(&pc)
    }

    /// Returns a mutable reference to data collected for a particular branch.
    /// Creates a new entry if one doesn't already exist.
    pub fn get_mut(&mut self, pc: usize) -> &mut BranchData {
        self.data.entry(pc).or_insert(BranchData::new())
    }

    /// Returns the number of unique observed branch instructions.
    pub fn num_unique_branches(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of branches that only occur once.
    pub fn num_single_occurence(&self) -> usize { 
        self.data.iter()
            .filter(|(_, entry)| entry.outcomes.len() == 1)
            .count()
    }

    /// Returns the number of branches that are always taken.
    pub fn num_always_taken(&self) -> usize {
        self.data.iter()
            .filter(|(_, entry)| entry.outcomes.is_always_taken())
            .count()
    }

    /// Returns the number of branches that are never taken.
    pub fn num_never_taken(&self) -> usize { 
        self.data.iter()
            .filter(|(_, entry)| entry.outcomes.is_never_taken())
            .count()
    }


    /// Return at most `n` branches whose hit rate falls below some threshold. 
    pub fn get_low_rate_branches(&self, n: usize) 
        -> Vec<(usize, &BranchData)> 
    {
        let iter = self.data.iter()
            .filter(|(_, s)| {
                s.occ > 100 && s.hit_rate() <= 0.55
            })
            .sorted_by(|x, y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() })
            .rev()
            .take(n);
        let res: Vec<(usize, &BranchData)> = iter.map(|(pc, s)| (*pc,s))
            .collect();
        res
    }
}


