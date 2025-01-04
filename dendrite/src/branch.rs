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
    pub fn from_bool(b: bool) -> Self { 
        match b { 
            true => Self::T,
            false => Self::N,
        }
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
    /// A direct conditional branch instruction.
    DirectBranch = BranchFlags::BRN_FLAG,

    /// A direct unconditional jump instruction.
    DirectJump   = BranchFlags::JMP_FLAG,

    /// An indirect unconditional jump instruction.
    IndirectJump = BranchFlags::JMP_FLAG | BranchFlags::IND_FLAG,

    /// A direct procedure call instruction.
    DirectCall   = BranchFlags::CALL_FLAG,

    /// An indirect procedure call instruction.
    IndirectCall = BranchFlags::CALL_FLAG | BranchFlags::IND_FLAG,

    /// A return instruction.
    Return       = BranchFlags::RET_FLAG | BranchFlags::IND_FLAG,
}
impl BranchKind { 
    const DIRECT_BRANCH: u32 = BranchFlags::BRN_FLAG;
    const DIRECT_JUMP: u32 = BranchFlags::JMP_FLAG;
    const DIRECT_CALL: u32 = BranchFlags::CALL_FLAG;
    const INDIRECT_CALL: u32 = BranchFlags::CALL_FLAG | BranchFlags::IND_FLAG;
    const INDIRECT_JUMP: u32 = BranchFlags::JMP_FLAG | BranchFlags::IND_FLAG;
    const RETURN: u32 = BranchFlags::RET_FLAG | BranchFlags::IND_FLAG;
}
impl From<u32> for BranchKind { 
    fn from(x: u32) -> Self { 
        match x & 0b01_1111 { 
            Self::DIRECT_BRANCH => Self::DirectBranch,
            Self::DIRECT_JUMP   => Self::DirectJump,
            Self::DIRECT_CALL   => Self::DirectCall,
            Self::INDIRECT_JUMP => Self::IndirectJump,
            Self::INDIRECT_CALL => Self::IndirectCall,
            Self::RETURN        => Self::Return,
            _ => unimplemented!("invalid flags? ({:05b})", x & 0b1_1111),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BranchFlags(pub u32);
impl BranchFlags { 

    const BRN_FLAG: u32   = (1 << 0);
    const JMP_FLAG: u32   = (1 << 1);
    const CALL_FLAG: u32  = (1 << 2);
    const RET_FLAG: u32   = (1 << 3);
    const IND_FLAG: u32   = (1 << 4);
    const TAKEN_FLAG: u32 = (1 << 5);

    /// 4-bit instruction length
    const ILEN_MASK: u32   = 0b1111_0000_0000_0000_0000_0000_0000_0000;

    pub fn ilen(&self) -> usize { 
        ((self.0 & Self::ILEN_MASK) >> 28) as usize
    }

    pub fn is_brn(&self) -> bool { self.0 & Self::BRN_FLAG != 0 }
    pub fn is_jmp(&self) -> bool { self.0 & Self::JMP_FLAG != 0 }
    pub fn is_call(&self) -> bool { self.0 & Self::CALL_FLAG != 0 }
    pub fn is_ret(&self) -> bool { self.0 & Self::RET_FLAG != 0 }
    pub fn is_direct(&self) -> bool { self.0 & Self::IND_FLAG == 0 }
    pub fn is_indirect(&self) -> bool { self.0 & Self::IND_FLAG != 0 }
    pub fn is_taken(&self) -> bool { self.0 & Self::TAKEN_FLAG != 0 }

    pub fn kind(&self) -> BranchKind { 
        self.0.try_into().unwrap()
    }

    pub fn new(kind: BranchKind, outcome: Outcome) -> Self { 
        let kbits = kind as u32;
        Self(kbits)
    }

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

    pub flags: BranchFlags,

    ///// The outcome evaluated for this branch
    //pub outcome: Outcome,

    ///// The type/kind of branch
    //pub kind: BranchKind,

    ///// Instruction length
    //pub ilen: u32,
}
impl BranchRecord {
    pub fn outcome(&self) -> Outcome { 
        Outcome::from_bool(self.flags.is_taken())
    }
    pub fn kind(&self) -> BranchKind { 
        self.flags.kind()
    }
    pub fn ilen(&self) -> usize { 
        self.flags.ilen()
    }

    /// Returns 'true' if this is a conditional instruction.
    pub fn is_conditional(&self) -> bool { 
        self.flags.is_brn()
    }

    /// Returns 'true' if this is an unconditional instruction.
    pub fn is_unconditional(&self) -> bool { 
        !self.flags.is_brn()
    }

    /// Returns 'true' if this instruction directly specifies the target. 
    pub fn is_direct(&self) -> bool { 
        self.flags.is_direct()
    }

    /// Returns 'true' if this instruction indirectly specifies the target. 
    pub fn is_indirect(&self) -> bool { 
        self.flags.is_indirect()
    }

    /// Returns 'true' if this is a "jump" instruction. 
    pub fn is_jump(&self) -> bool { 
        self.flags.is_jmp()
    }

    /// Returns 'true' if this is a "call" or "return". 
    pub fn is_procedural(&self) -> bool { 
        self.flags.is_call() || self.flags.is_ret()
    }
}


