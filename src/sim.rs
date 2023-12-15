//! Tools for synthesizing execution traces. 
//!
//! Intermediate Representation
//! ===========================
//!
//! In this intermediate language, we really only need the ability to:
//!
//! - Set the program counter to a particular value
//! - Emit a control-flow instruction whose program counter, target address,
//!   and outcomes are all statically pre-determined
//!
//! An interpreter over this language should be sufficient to unroll our 
//! IR into something like a trace.
//!

use std::collections::*;
use crate::direction::Outcome;

/// A pre-determined pattern of branch outcomes to-be-associated with a 
/// control-flow instruction in the IR. 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchPattern {
    AlwaysTaken,
    NeverTaken,
    TakenPeriodic(usize),
    NotTakenPeriodic(usize),
    Pattern(&'static [Outcome]),
}

/// Representing a branch target in the IR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IRReloc { 
    /// An unresolved pointer to some [IRInst]. 
    Label(Label),
    /// A resolved target address.
    Address(usize),
}

/// An instruction in the IR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IRInst {
    /// A conditional branch instruction.
    Branch(IRReloc),

    /// An unconditional branch instruction.
    Jump(IRReloc),

    /// Increment the program counter by some value. 
    Pad(usize),

    /// Increment the program counter until the value is aligned to some 
    /// power of two. 
    PadAlign(usize),

    /// Terminate the program
    Terminate,
}
impl IRInst {
    /// Returns the number of "virtual bytes" inhabited by this instruction.
    pub fn size(&self, pc: usize) -> usize { 
        match self { 
            Self::Branch(_) => 1,
            Self::Jump(_) => 1,
            Self::Pad(size) => *size,
            Self::PadAlign(aln) => {
                assert!(aln.is_power_of_two());
                let mask = aln - 1;
                let tgt  = (pc + mask) & !mask;
                let diff = tgt - pc;
                diff
            },
            Self::Terminate => 0,
        }
    }
}



/// An identifier for a particular [IRInst] within the program. 
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(usize);
impl Label {
    pub fn new(id: usize) -> Self { Self(id) }
    pub fn id(&self) -> usize { self.0 }
}

pub struct Emitter { 
    /// The set of instructions. 
    data: Vec<IRInst>,
    /// The base address of the program. 
    base: usize,
    /// A counter used to allocate labels.
    next_label: usize,
    /// A map from labels to offsets within `data`. 
    labels: BTreeMap<usize, usize>,
}
impl Emitter { 
    pub fn new(base: usize) -> Self { 
        Self { 
            data: Vec::new(),
            next_label: 0,
            base,
            labels: BTreeMap::new(),
        }
    }

    /// Create a new label.
    pub fn create_label(&mut self) -> Label {
        let res = Label::new(self.next_label);
        self.next_label += 1;
        res
    }

    /// Bind a label to the current point in the program.
    pub fn bind_label(&mut self, label: Label) {
        let off = self.data.len();
        self.labels.insert(label.id(), off);
    }

    /// Emit a conditional branch to the provided [Label].
    pub fn branch_to_label(&mut self, tgt: Label) {
        self.data.push(IRInst::Branch(IRReloc::Label(tgt)));
    }

    /// Emit an unconditional branch to the provided [Label].
    pub fn jump_to_label(&mut self, tgt: Label) {
        self.data.push(IRInst::Jump(IRReloc::Label(tgt)));
    }


    /// Increment the program counter by some value.
    pub fn pad(&mut self, size: usize) {
        self.data.push(IRInst::Pad(size));
    }

    /// Increment and align the program counter to some power of two.
    pub fn pad_align(&mut self, aln: usize) {
        self.data.push(IRInst::PadAlign(aln));
    }

    /// Compute and return the list of program counter values associated with
    /// each instruction. 
    fn resolve_pc_table(&self) -> Vec<usize> {
        let mut pc = self.base;
        let mut res = Vec::new();
        for inst in self.data.iter() {
            res.push(pc);
            pc += inst.size(pc);
        }
        res
    }

    /// Resolve all labels into target addresses. 
    fn resolve_relocations(&mut self) {
        let pc_table = self.resolve_pc_table();
        for inst in self.data.iter_mut() {
            match inst { 
                IRInst::Jump(ref mut reloc) |
                IRInst::Branch(ref mut reloc) => {
                    if let IRReloc::Label(lab) = reloc { 
                        let idx = *self.labels.get(&lab.id()).unwrap();
                        let tgt_addr = pc_table[idx];
                        *reloc = IRReloc::Address(tgt_addr);
                    }
                },
                _ => {},
            }
        }
    }

    pub fn emit(&mut self) -> Program {
        self.resolve_relocations();

        let mut data: Vec<Branch> = Vec::new();
        let mut pc = self.base;
        for inst in self.data.iter() {
            match inst { 
                IRInst::Branch(IRReloc::Label(_)) |
                IRInst::Jump(IRReloc::Label(_)) => {
                    unreachable!("Unresolved label");
                },
                IRInst::Branch(IRReloc::Address(tgt)) => {
                    data.push(Branch::new(pc, *tgt, BranchKind::Conditional));
                },
                IRInst::Jump(IRReloc::Address(tgt)) => {
                    data.push(Branch::new(pc, *tgt, BranchKind::Unconditional));
                },
                _ => {},
            }
            pc += inst.size(pc);
        }

        Program::new(self.base, data)
    }
}

/// A type of [Branch]. 
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchKind {
    Conditional, Unconditional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branch { 
    addr: usize,
    tgt: usize,
    kind: BranchKind,
}
impl Branch {
    pub fn new(addr: usize, tgt: usize, kind: BranchKind) -> Self { 
        Self { addr, tgt, kind }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    base: usize,
    data: Vec<Branch>,
}
impl Program {
    pub fn new(base: usize, data: Vec<Branch>) -> Self { 
        Self { 
            base,
            data,
        }
    }
}


#[cfg(test)]
mod test { 
    use super::*;
    use crate::direction::Outcome;
    #[test]
    fn foo() {
        let mut e = Emitter::new(0x1000_0000);
        let start = e.create_label();
        let next = e.create_label();

        e.bind_label(start);
        e.branch_to_label(next);

        e.pad_align(0x0000_1000);
        e.bind_label(next);

        e.pad_align(0x0000_0100);
        e.branch_to_label(start);

        let p = e.emit();
        println!("{:x?}", p);

    }
}


