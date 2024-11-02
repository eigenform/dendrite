//! Types for representing branches and branch outcomes. 

use std::collections::*;
use bitvec::prelude::*;

/// A branch outcome. 
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome { 
    /// Not taken
    N = 0,
    /// Taken
    T = 1 
}

impl Outcome { 
    pub fn vec_from_bitvec(bits: &BitVec) -> Vec<Self> {
        bits.iter().map(|b| (*b).into()).collect()
    }
}

impl std::fmt::Debug for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self { 
            Self::T => "t",
            Self::N => "n",
        };
        write!(f, "{}", s)
    }
}

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

/// Representing different kinds of branch/control-flow instructions.
///
/// NOTE: This enum is kept in-sync *manually* with headers in the DynamoRIO
/// client (see `./dynamorio/src/dendrite.h`).
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BranchKind {
    //Invalid      = 0x00,

    /// A direct conditional branch instruction.
    DirectBranch = 0x10,

    /// A direct unconditional jump instruction.
    DirectJump   = 0x20,

    /// An indirect unconditional jump instruction.
    IndirectJump = 0x21,

    /// A direct procedure call instruction.
    DirectCall   = 0x40, 

    /// An indirect procedure call instruction.
    IndirectCall = 0x41, 

    /// A return instruction.
    Return       = 0x81,
}


/// A record of branch execution. 
///
/// NOTE: The layout of this struct is kept in-sync *manually* with headers
/// in the DynamoRIO client (see `./dynamorio/src/dendrite.h`).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
impl BranchRecord {
    /// Returns 'true' if this is a conditional instruction.
    pub fn is_conditional(&self) -> bool { 
        self.kind == BranchKind::DirectBranch
    }

    /// Returns 'true' if this is an unconditional instruction.
    pub fn is_unconditional(&self) -> bool { 
        self.kind != BranchKind::DirectBranch
    }

    /// Returns 'true' if this instruction directly specifies the target. 
    pub fn is_direct(&self) -> bool { 
        match self.kind {
            BranchKind::DirectBranch |
            BranchKind::DirectJump |
            BranchKind::DirectCall => true,
            _ => false,
        }
    }

    /// Returns 'true' if this instruction indirectly specifies the target. 
    pub fn is_indirect(&self) -> bool { 
        match self.kind {
            BranchKind::IndirectJump |
            BranchKind::IndirectCall |
            BranchKind::Return => true,
            _ => false,
        }
    }

    /// Returns 'true' if this is a "jump" instruction. 
    pub fn is_jump(&self) -> bool { 
        match self.kind { 
            BranchKind::DirectJump |
            BranchKind::IndirectJump => true,
            _ => false,

        }
    }

    /// Returns 'true' if this is a "call" or "return". 
    pub fn is_procedural(&self) -> bool { 
        match self.kind {
            BranchKind::DirectCall | 
            BranchKind::IndirectCall | 
            BranchKind::Return => true,
            _ => false,
        }
    }
}


