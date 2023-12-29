
use std::collections::*;
use bitvec::prelude::*;

/// A branch outcome. 
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { N = 0, T = 1 }
impl std::ops::Not for Outcome { 
    type Output = Self;
    fn not(self) -> Self { 
        match self { 
            Self::N => Self::T,
            Self::T => Self::N,
        }
    }
}
impl From<bool> for Outcome {
    fn from(x: bool) -> Self { 
        match x { 
            true => Self::T,
            false => Self::N 
        }
    }
}
impl Into<bool> for Outcome {
    fn into(self) -> bool { 
        match self { 
            Self::T => true,
            Self::N => false,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BranchKind {
    Invalid      = 0x00,
    DirectBranch = 0x10,
    DirectJump   = 0x20,
    IndirectJump = 0x21,
    DirectCall   = 0x40, 
    IndirectCall = 0x41, 
    Return       = 0x81,
}


/// A record of branch execution. 
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BranchRecord { 
    /// The program counter value for this branch
    pub pc: usize,

    /// The target address evaluated for this branch
    pub tgt: usize,

    /// The outcome evaluated for this branch
    pub outcome: Outcome,

    /// The type/kind of branch
    pub kind: BranchKind,
}


/// Container for recording statistics while evaluating some model.
pub struct BranchStats { 
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

    pub fn hit_rate(&self) -> f64 {
        self.global_hits as f64 / self.global_brns as f64
    }

    pub fn global_hits(&self) -> usize { self.global_hits }
    pub fn global_miss(&self) -> usize { self.global_brns - self.global_hits }
    pub fn global_brns(&self) -> usize { self.global_brns }

    pub fn get(&self, pc: usize) -> Option<&BranchData> {
        self.data.get(&pc)
    }

    pub fn get_mut(&mut self, pc: usize) -> &mut BranchData {
        self.data.entry(pc).or_insert(BranchData::new())
    }

    pub fn num_unique_branches(&self) -> usize { 
        self.data.len()
    }
}

pub struct BranchData { 
    pub occ: usize,
    pub hits: usize,
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
    pub fn hit_rate(&self) -> f64 {
        self.hits as f64 / self.occ as f64
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


