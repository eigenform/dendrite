
pub mod assembler;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::branch::*;

pub struct BinaryTraceSet {
    /// A list of filenames
    pub files: Vec<String>,

    pub cur: usize,
    pub next: usize,
}
impl BinaryTraceSet {
    pub fn new() -> Self { 
        Self { 
            files: Vec::new(),
            cur: 0,
            next: 1,
        }
    }

    pub fn new_from_slice(strings: &[String]) -> Self {
        let mut files = Vec::new();
        files.extend_from_slice(strings);
        Self { 
            files,
            cur: 0,
            next: 1,
        }
    }

    pub fn add_file(&mut self, s: impl ToString) {
        self.files.push(s.to_string());
    }
}
impl Iterator for BinaryTraceSet {
    type Item = BinaryTrace;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.files.len() {
            return None;
        } else {
            let cur = self.cur;
            let name = Path::new(&self.files[cur])
                .file_name().unwrap()
                .to_str().unwrap();
            let trace = BinaryTrace::from_file(&self.files[cur], name);
            self.cur += 1;
            Some(trace)
        }
    }
}


/// A trace generated with the 'dendrite' client for DynamoRIO. 
pub struct BinaryTrace {
    pub data: Vec<u8>,
    pub name: String,
    /// Number of entries
    num_entries: usize,
}
impl BinaryTrace {

    /// Create a [BinaryTrace] from a file.
    /// NOTE: We aren't validating input at all
    pub fn from_file(path: &str, name: &str) -> Self {
        let mut f = File::open(path).unwrap();
        let len = std::fs::metadata(path).unwrap().len() as usize;
        assert!(len % std::mem::size_of::<BranchRecord>() == 0);

        let num_entries = len / std::mem::size_of::<BranchRecord>();
        let mut data = vec![0; len];
        f.read(&mut data).unwrap();
        Self { 
            data, 
            num_entries,
            name: name.to_string(),
        }
    }

    /// Return the number of records
    pub fn num_entries(&self) -> usize { self.num_entries }

    pub fn name(&self) -> &str { &self.name }

    /// Return a truncated slice of records
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

    /// Return a slice of records.
    pub fn as_slice(&self) -> &[BranchRecord] {
        unsafe { 
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const BranchRecord,
                self.num_entries
            )
        }
    }

}


