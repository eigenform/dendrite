
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


pub struct BranchStats { 
    pub data: BTreeMap<usize, BranchData>,
    pub hits: usize,
    pub num_branches: usize,
}
impl BranchStats {
    pub fn new() -> Self {
        Self { 
            data: BTreeMap::new(),
            hits: 0,
            num_branches: 0,
        }
    }

    pub fn get(&self, pc: usize) -> Option<&BranchData> {
        self.data.get(&pc)
    }

    pub fn get_mut(&mut self, pc: usize) -> &mut BranchData {
        self.data.entry(pc).or_insert(BranchData::new())
    }

    pub fn num_branches(&self) -> usize { 
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
}


