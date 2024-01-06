//! Helpers for collecting statistics.

use std::collections::*;
use crate::branch::*;
use bitvec::prelude::*;
use itertools::*;

/// Container for recording simple statistics while evaluating some model.
pub struct BranchStats {
    /// Per-branch statistics (indexed by program counter value).
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
        data.pat.push(outcome.into());
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
            .filter(|(pc, entry)| entry.pat.len() == 1)
            .count()
    }

    /// Returns the number of branches that are always taken
    pub fn num_always_taken(&self) -> usize {
        self.data.iter()
            .filter(|(pc, entry)| { entry.pat.iter().all(|o| *o == true) })
            .count()
    }

    /// Returns the number of branches that are never taken
    pub fn num_never_taken(&self) -> usize { 
        self.data.iter()
            .filter(|(pc, entry)| { entry.pat.iter().all(|o| *o == false) })
            .count()
    }


    pub fn get_common_branches(&self, n: usize) -> Vec<(usize, &BranchData)> {
        let iter = self.data.iter()
            .sorted_by(|x, y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() })
            .rev()
            .take(n);
        let res: Vec<(usize, &BranchData)> = iter.map(|(pc, s)| (*pc, s))
            .collect();
        res
    }

    pub fn get_low_rate_branches(&self, n: usize) 
        -> Vec<(usize, &BranchData)> 
    {
        let iter = self.data.iter()
            .filter(|(_, s)| {
                s.occ > 100 && s.hit_rate() <= 0.55
            })
            //.sorted_by(|x, y| { 
            //    x.1.hit_rate().partial_cmp(&y.1.hit_rate()).unwrap()
            //})
            .sorted_by(|x, y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() })
            .rev()
            .take(n);
        let res: Vec<(usize, &BranchData)> = iter.map(|(pc, s)| (*pc,s))
            .collect();
        res
    }

}

/// Container for per-branch statistics.
pub struct BranchData {
    /// Number of times this branch was encountered.
    pub occ: usize,

    /// Number of correct predictions for this branch.
    pub hits: usize,

    /// Record of all observed outcomes for this branch.
    pub pat: BitVec,
}
impl BranchData {
    pub fn new() -> Self {
        Self {
            occ: 0,
            hits: 0,
            pat: BitVec::new(),
        }
    }

    /// Return the hit rate for this branch.
    pub fn hit_rate(&self) -> f64 {
        self.hits as f64 / self.occ as f64
    }

    pub fn is_always_taken(&self) -> bool {
        self.pat.count_ones() == self.pat.len()
    }

    pub fn is_never_taken(&self) -> bool {
        self.pat.count_zeros() == self.pat.len()
    }

    pub fn times_taken(&self) -> usize { 
        self.pat.count_ones()
    }


    // NOTE: Remember that this isn't too useful apart from telling you
    // whether some sequence of outcomes is mixed or uniform.
    pub fn shannon_entropy(&self) -> f64 {
        let n   = self.pat.len() as f64;
        let n_t = self.pat.count_ones();
        let n_f = self.pat.count_zeros();

        let p_t = (n_t as f64) / n;
        let p_f = (n_f as f64) / n;

        let res = -(p_t * p_t.log2() + p_f * p_f.log2());
        if res.is_nan() { 0.0 } else { res }
    }
}


