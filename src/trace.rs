
pub mod assembler;

use crate::branch::*;

/// A trace generated with the 'dendrite' client for DynamoRIO. 
pub struct BinaryTrace {
    pub data: Vec<u8>,
    /// Number of entries
    num_entries: usize,
}
impl BinaryTrace {
    // NOTE: We aren't validating input at all
    pub fn from_file(path: &str) -> Self {
        use std::fs::File;
        use std::io::Read;
        let mut f = File::open(path).unwrap();
        let len = std::fs::metadata(path).unwrap().len() as usize;
        assert!(len % std::mem::size_of::<BranchRecord>() == 0);

        let num_entries = len / std::mem::size_of::<BranchRecord>();
        let mut data = vec![0; len];
        f.read(&mut data).unwrap();
        Self { data, num_entries }
    }

    pub fn num_entries(&self) -> usize { self.num_entries }

    pub fn as_slice_trunc(&self, limit: usize) -> &[BranchRecord] {
        let req_entries = if self.num_entries > limit {
            limit
        } else {
            self.num_entries
        };

        unsafe { 
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const BranchRecord,
                req_entries
            )
        }
    }


    pub fn as_slice(&self) -> &[BranchRecord] {
        unsafe { 
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const BranchRecord,
                self.num_entries
            )
        }
    }

}


