
use std::collections::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relocation { 
    Label(Label),
    /// A signed offset to the target address
    Offset(i64),
    /// A target address
    Address(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchKind { 
    Conditional,
    Unconditional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IRInst {
    Branch(BranchKind, Relocation),
    Jump(Relocation),
    Pad(usize),
    PadAlign(usize),
}
impl IRInst {
    pub fn size(&self, pc: usize) -> usize { 
        match self { 
            Self::Branch(_, _) => 1,
            Self::Jump(_) => 1,
            Self::Pad(size) => *size,
            Self::PadAlign(aln) => {
                assert!(aln.is_power_of_two());
                let mask = aln - 1;
                let tgt  = (pc + mask) & !mask;
                let diff = tgt - pc;
                diff
            },
        }
    }
}



/// A pointer to an [IRInst].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(usize);
impl Label {
    pub fn new(id: usize) -> Self { Self(id) }
    pub fn index(&self) -> usize { self.0 }
}

pub struct Emitter { 
    data: Vec<IRInst>,
    base: usize,
    next_label: usize,
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
        self.labels.insert(label.0, off);
    }

    pub fn branch_to_label(&mut self, tgt: Label) {
        self.data.push(IRInst::Branch(
            BranchKind::Conditional, Relocation::Label(tgt)
        ));
    }

    pub fn jump_to_label(&mut self, tgt: Label) {
        self.data.push(IRInst::Branch(
            BranchKind::Unconditional, Relocation::Label(tgt)
        ));
    }


    pub fn pad(&mut self, size: usize) {
        self.data.push(IRInst::Pad(size));
    }
    pub fn pad_align(&mut self, aln: usize) {
        self.data.push(IRInst::PadAlign(aln));
    }


    fn resolve_relocations(&mut self) {
        let mut pc = self.base;
        let mut addresses = Vec::new();
        for inst in self.data.iter() {
            addresses.push(pc);
            pc += inst.size(pc);
        }

        for (cur, inst) in self.data.iter_mut().enumerate() {
            match inst { 
                IRInst::Branch(_, ref mut reloc) => {
                    if let Relocation::Label(lab) = reloc { 
                        // The index of the target instruction
                        let idx = *self.labels.get(&lab.0).unwrap();
                        let tgt_addr = addresses[idx];
                        *reloc = Relocation::Address(tgt_addr);
                    }
                },
                _ => {},
            }

            pc += inst.size(pc);
        }
    }

    pub fn emit(&mut self) -> Program {
        println!("{:?}", self.data);
        println!("labels: {:?}", self.labels);
        self.resolve_relocations();

        let mut data: Vec<Branch> = Vec::new();
        let mut pc = self.base;
        for inst in self.data.iter() {
            match inst { 
                IRInst::Branch(_, Relocation::Label(_)) => {
                    unreachable!("Unresolved label");
                },
                IRInst::Branch(_, Relocation::Address(tgt)) => {
                    let branch = Branch::new(pc, *tgt);
                    data.push(branch);
                },
                _ => {},
            }
            pc += inst.size(pc);
        }

        Program::new(self.base, data)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Branch { 
    addr: usize,
    tgt: usize,
}
impl Branch {
    pub fn new(addr: usize, tgt: usize) -> Self { 
        Self { addr, tgt }
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


