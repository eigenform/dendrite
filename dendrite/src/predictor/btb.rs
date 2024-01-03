
use crate::predictor::*;
use crate::branch::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimpleBTBEntry {
    /// Cached target address for this branch
    pub tgt: usize,
    /// The type of branch
    pub kind: BranchKind,
    /// Is this entry valid?
    pub valid: bool,
    /// Use the predictor for the associated [BranchKind]
    pub alt: bool,
}
impl SimpleBTBEntry {
    pub fn new() -> Self { 
        Self { 
            tgt: 0, 
            kind: BranchKind::Invalid, 
            valid: false,
            alt: false,
        }
    }

    pub fn matches(&self, record: &BranchRecord) -> bool {
        assert!(record.kind != BranchKind::Invalid);
        if !self.valid { return false; }
        if self.kind != record.kind { return false; }
        match self.kind {
            BranchKind::Invalid => { return false; },
            BranchKind::DirectJump |
            BranchKind::DirectCall |
            BranchKind::DirectBranch => {
                if self.tgt != record.tgt {
                    return false;
                }
            },
            BranchKind::IndirectJump |
            BranchKind::IndirectCall |
            BranchKind::Return => {
            },
        }

        self.tgt == record.tgt &&
        self.kind == record.kind &&
        self.valid
    }

    pub fn target(&self) -> usize { self.tgt }
    pub fn kind(&self) -> BranchKind { self.kind }
    pub fn valid(&self) -> bool { self.valid }
    pub fn alt(&self) -> bool { self.alt }
}

pub struct SimpleBTB {
    size: usize,
    data: Vec<SimpleBTBEntry>,
}
impl SimpleBTB {
    pub fn new(size: usize) -> Self {
        assert!(size.is_power_of_two());
        Self { 
            size,
            data: vec![SimpleBTBEntry::new(); size],
        }
    }
}

//impl PredictorTable for SimpleBTB {
//    /// The type of input to the table used to form an index.
//    type Input<'a> = usize;
//
//    type Index = usize;
//
//    /// The type of entry in the table.
//    type Entry = SimpleBTBEntry;
//
//    /// Returns the number of entries in the table.
//    fn size(&self) -> usize { self.size }
//
//    /// Given some input, return the corresponding index into the table. 
//    //fn get_index(&self, input: Self::Input) -> usize {
//    //    (self.index_fn)(input) & self.index_mask()
//    //}
//
//    /// Returns a reference to an entry in the table.
//    fn get_entry(&self, idx: Self::Index) -> &Self::Entry {
//        &self.data[idx]
//    }
//
//    /// Returns a mutable reference to an entry in the table.
//    fn get_entry_mut(&mut self, idx: Self::Index) -> &mut Self::Entry {
//        &mut self.data[idx]
//    }
//}
