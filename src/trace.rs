
use std::collections::*;
use crate::direction::Outcome;

/// An identifier for a particular [EmitterOp].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Label(usize);
impl Label {
    pub fn new(id: usize) -> Self { Self(id) }
    pub fn id(&self) -> usize { self.0 }
}

#[derive(Debug)]
pub struct LabelDb {
    data: Vec<Option<usize>>,
    next: usize,
}
impl LabelDb { 
    pub fn new() -> Self { 
        Self { data: Vec::new(), next: 0 }
    }
    pub fn alloc(&mut self) -> Label {
        let res = Label::new(self.next);
        self.data.push(None);
        self.next += 1;
        res
    }
    pub fn define(&mut self, label: &Label, idx: usize) {
        self.data[label.id()] = Some(idx);
    }
    pub fn resolve(&self, label: &Label) -> Option<usize> { 
        self.data[label.id()]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitterLoc { 
    /// A label resolved into an index of some other [EmitterOp] at compile-time.
    Label(Label),

    /// An index pointing to some other [EmitterOp]. 
    Index(usize),
}
impl EmitterLoc {
    pub fn get_index(&self) -> usize { 
        if let Self::Index(idx) = self {
            *idx
        } else {
            panic!("Unresolved label {:?}", self);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchTarget { 
    /// A single fixed target location
    Direct(EmitterLoc),

    /// A list of target locations
    Indirect(Vec<EmitterLoc>),
}


/// A pre-determined pattern of outcomes associated with a conditional branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchPattern {
    /// A branch whose outcome is always 'taken'.
    AlwaysTaken,

    /// A branch whose outcome is always 'not-taken'.
    NeverTaken,

    /// A branch whose outcome is only periodically "taken".
    /// Otherwise, the branch is "not-taken" by default. 
    TakenPeriodic(usize),

    /// A branch whose outcome is only periodically "not-taken".
    /// Otherwise, the branch is "taken" by default.
    NotTakenPeriodic(usize),

    /// A branch with an arbitrary pattern of outcomes. 
    Pattern(&'static [Outcome]),
}

/// An instruction in the IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitterOp {
    /// A conditional branch instruction.
    /// Consists of a single target, and some pattern of outcomes.
    Branch(BranchTarget, BranchPattern),

    /// An unconditional branch instruction with a single target. 
    Jump(BranchTarget),
}
impl EmitterOp {
    /// Returns the number of "bytes" that correspond to this instruction.
    pub fn size(&self) -> usize { 
        match self { 
            Self::Branch(_, _) => 1,
            Self::Jump(_) => 1,
        }
    }

    /// Given some counter, generate a branch outcome. 
    pub fn outcome(&self, ctr: usize) -> Outcome {
        match self { 
            EmitterOp::Jump(_) => Outcome::T,
            EmitterOp::Branch(_, pat) => match pat { 
                BranchPattern::Pattern(p) => p[ctr % p.len()],
                BranchPattern::NeverTaken => Outcome::N,
                BranchPattern::AlwaysTaken => Outcome::N,
                BranchPattern::TakenPeriodic(p) => {
                    if ctr % p == (p - 1) { Outcome::T } else { Outcome::N }
                },
                BranchPattern::NotTakenPeriodic(p) => {
                    if ctr % p == (p - 1) { Outcome::N } else { Outcome::T }
                },
            }
        }
    }

    pub fn target_loc(&self, ctr: usize) -> &EmitterLoc {
        match self {
            EmitterOp::Jump(BranchTarget::Direct(loc)) |
            EmitterOp::Branch(BranchTarget::Direct(loc), _) => loc,

            EmitterOp::Jump(BranchTarget::Indirect(locs)) |
            EmitterOp::Branch(BranchTarget::Indirect(locs), _) => {
                unimplemented!();
            },
        }
    }

}




#[derive(Debug)]
pub struct TraceEmitter { 
    /// The list of [EmitterOp]s. 
    ops: Vec<EmitterOp>,

    /// The list of program counter values corresponding to each [EmitterOp].
    pcs: Vec<usize>,

    ctr: Vec<usize>,

    /// The initial address/program counter value.
    base: usize,

    /// State tracking the program counter value during assembly.
    cursor: usize,

    /// Map from some [Label] to an [EmitterOp] index.
    labels: LabelDb,
}

impl TraceEmitter { 
    /// Create a new emitter. 
    pub fn new(base: usize) -> Self { 
        Self { 
            ops: Vec::new(),
            pcs: Vec::new(),
            ctr: Vec::new(),
            base,
            cursor: base,
            labels: LabelDb::new(),
        }
    }

    /// Create a new label.
    pub fn create_label(&mut self) -> Label {
        self.labels.alloc()
    }

    /// Bind a label to the current point in the program.
    pub fn bind_label(&mut self, label: Label) {
        let off = self.ops.len();
        self.labels.define(&label, off);
    }

    /// Create all of the state we need to simulate this [EmitterOp].
    fn push_op(&mut self, op: EmitterOp) {
        let op_size = op.size();
        self.ops.push(op);
        self.pcs.push(self.cursor);
        self.ctr.push(0);
        self.cursor += op_size;
    }

    /// Emit a conditional direct branch to the provided [Label].
    pub fn branch_to_label(&mut self, tgt: Label, pat: BranchPattern) {
        self.push_op(EmitterOp::Branch(
            BranchTarget::Direct(EmitterLoc::Label(tgt)), 
            pat
        ));
    }

    /// Emit an unconditional direct branch to the provided [Label].
    pub fn jump_to_label(&mut self, tgt: Label) {
        self.push_op(EmitterOp::Jump(
            BranchTarget::Direct(EmitterLoc::Label(tgt))
        ));
    }

    /// Increment the program counter by some value.
    pub fn pad(&mut self, len: usize) { 
        self.cursor = self.cursor + len; 
    }

    /// Increment and align the program counter to some power of two.
    pub fn pad_align(&mut self, aln: usize) {
        assert!(aln.is_power_of_two());
        let mask = aln - 1;
        let tgt  = (self.cursor + mask) & !mask;
        self.cursor = tgt - self.cursor;
    }

    /// Explicitly set the program counter to a particular value. 
    pub fn pad_until(&mut self, next_pc: usize) {
        assert!(next_pc > self.cursor);
        self.cursor = next_pc;
    }
}


impl TraceEmitter {
    /// Rewrite occurences of [EmitterLoc::Label] into [EmitterLoc::Index].
    /// This function will panic when encountering an undefined label.
    fn rewrite_labels(&mut self) {
        // Rewrite direct targets
        let locs = self.ops.iter_mut().filter_map(|op| { match op {
            EmitterOp::Branch(BranchTarget::Direct(ref mut loc), _) |
            EmitterOp::Jump(BranchTarget::Direct(ref mut loc)) => Some(loc),
            _ => None,
        }});

        for loc in locs {
            if let EmitterLoc::Label(lab) = loc {
                if let Some(idx) = self.labels.resolve(lab) {
                    *loc = EmitterLoc::Index(idx);
                } else {
                    panic!("Undefined label {:?}", lab);
                }
            }
        }

        // Rewrite indirect targets
        let indir_locs = self.ops.iter_mut().filter_map(|op| { match op {
            EmitterOp::Branch(BranchTarget::Indirect(ref mut locs), _) |
            EmitterOp::Jump(BranchTarget::Indirect(ref mut locs)) => Some(locs),
            _ => None,
        }});

        for locs in indir_locs {
            for loc in locs {
                if let EmitterLoc::Label(lab) = loc {
                    if let Some(idx) = self.labels.resolve(lab) {
                        *loc = EmitterLoc::Index(idx);
                    } else {
                        panic!("Undefined label {:?}", lab);
                    }
                }
            }
        }
    }


    pub fn simulate_for(&mut self, max_iters: usize) -> Vec<BranchRecord> {
        self.rewrite_labels();

        let mut res = Vec::new();
        let mut cur = 0;
        let mut iter = 0;

        loop {
            if iter >= max_iters { break; }
            if cur >= self.ops.len() { break; }

            let op  = &self.ops[cur];
            let ctr = self.ctr[cur];
            let pc  = self.pcs[cur];

            let outcome = op.outcome(ctr);
            let tgt_loc = op.target_loc(ctr);
            let tgt_idx = tgt_loc.get_index();
            let tgt     = self.pcs[tgt_idx];

            // Increment the counter for this branch
            self.ctr[cur] += 1;

            if outcome == Outcome::T {
                cur = tgt_idx;
            } else {
                cur += op.size();
            }

            res.push(BranchRecord { pc, tgt, outcome });

            iter += 1;
        }
        res
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BranchRecord { 
    pub pc: usize,
    pub tgt: usize,
    pub outcome: Outcome,
}


