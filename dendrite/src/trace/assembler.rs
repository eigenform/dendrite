
use crate::branch::*;

pub struct SyntheticTrace { 
    pub data: Vec<BranchRecord>,
}

/// An identifier for a particular [EmitterOp].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Label(usize);
impl Label {
    pub fn new(id: usize) -> Self { Self(id) }
    pub fn id(&self) -> usize { self.0 }
}

/// A map from labels to indexes. 
#[derive(Debug)]
pub struct LabelDb {
    data: Vec<Option<usize>>,
    next: usize,
}
impl LabelDb { 
    pub fn new() -> Self { 
        Self { data: Vec::new(), next: 0 }
    }

    /// Allocate a new label
    pub fn alloc(&mut self) -> Label {
        let res = Label::new(self.next);
        self.data.push(None);
        self.next += 1;
        res
    }

    /// Bind a label to some index
    pub fn define(&mut self, label: &Label, idx: usize) {
        self.data[label.id()] = Some(idx);
    }

    /// Resolve a label
    pub fn resolve(&self, label: &Label) -> Option<usize> { 
        self.data[label.id()]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitterLoc { 
    /// A label to be resolved into an index at compile-time.
    Label(Label),

    /// An index pointing to some [EmitterOp]. 
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

    /// Return the [BranchKind] for this op. 
    pub fn kind(&self) -> BranchKind {
        match self {
            EmitterOp::Jump(BranchTarget::Direct(_)) 
                => BranchKind::DirectJump,
            EmitterOp::Jump(BranchTarget::Indirect(_)) 
                => BranchKind::IndirectJump,
            EmitterOp::Branch(BranchTarget::Direct(_), _) 
                => BranchKind::DirectBranch,
            EmitterOp::Branch(BranchTarget::Indirect(_), _) 
                => unimplemented!(),
        }
    }

    /// Generate a branch outcome with the provided value.
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

    /// Generate a branch target with the provided value.
    pub fn target_loc(&self, _ctr: usize) -> &EmitterLoc {
        match self {
            EmitterOp::Jump(BranchTarget::Direct(loc)) |
            EmitterOp::Branch(BranchTarget::Direct(loc), _) => loc,

            EmitterOp::Jump(BranchTarget::Indirect(_locs)) |
            EmitterOp::Branch(BranchTarget::Indirect(_locs), _) => {
                unimplemented!();
            },
        }
    }

}



/// Used to assemble and compile a trace. 
#[derive(Debug)]
pub struct TraceAssembler { 
    /// The list of [EmitterOp]s. 
    ops: Vec<EmitterOp>,

    /// The list of program counter values corresponding to each [EmitterOp].
    pcs: Vec<usize>,

    /// The initial address/program counter value.
    base: usize,

    /// State tracking the program counter value during assembly.
    cursor: usize,

    /// Map from some [Label] to an [EmitterOp] index.
    labels: LabelDb,
}

impl TraceAssembler { 
    /// Create a new emitter. 
    pub fn new(base: usize) -> Self { 
        Self { 
            ops: Vec::new(),
            pcs: Vec::new(),
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
        println!("tgt={:08x} cur={:08x}", tgt, self.cursor);
        self.cursor = self.cursor + (tgt - self.cursor);
    }

    /// Explicitly set the program counter to a particular value. 
    pub fn pad_until(&mut self, next_pc: usize) {
        assert!(next_pc > self.cursor);
        self.cursor = next_pc;
    }
}


impl TraceAssembler {
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


    /// Unroll this program into a trace.
    pub fn compile(&mut self, max_iters: usize) -> SyntheticTrace {
        self.rewrite_labels();

        let num_ops = self.ops.len();
        let mut ctr = vec![0; num_ops];
        let mut data = Vec::new();
        let mut cur = 0;
        let mut iter = 0;

        'main: loop {
            if (iter >= max_iters) || (cur >= num_ops) {
                break 'main;
            }

            let op  = &self.ops[cur];
            let pc  = self.pcs[cur];

            let outcome = op.outcome(ctr[cur]);
            let tgt_loc = op.target_loc(ctr[cur]);
            let tgt_idx = tgt_loc.get_index();
            let tgt     = self.pcs[tgt_idx];
            let kind    = op.kind();
            ctr[cur] += 1;

            let record  = BranchRecord { pc, tgt, outcome, kind };
            data.push(record);

            // Go to the next instruction
            iter += 1;
            cur = match outcome {
                Outcome::T => tgt_idx,
                Outcome::N => cur + op.size(),
            };
        }

        SyntheticTrace { data }
    }
}


