#![feature(specialization)]

use dendrite::*;
use itertools::*;
use std::env;
use std::collections::*;
use bitvec::prelude::*;


#[derive(Clone, PartialEq, Eq)]
pub struct Run<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    pub len: usize,
    pub data: Vec<T>,
}

impl <T> std::fmt::Debug for Run<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    default fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.len == 1 {
            write!(f, "{:?}", self.data[0])
        } else { 
            write!(f, "{}{:?}", self.len, self.data)
        }
    }
}

impl std::fmt::Debug for Run<Outcome> 
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{:?}", self.len, self.data[0])
    }
}


#[derive(Clone, PartialEq, Eq)]
pub struct BaseRunPair {
    pub basis: Outcome,
    pub head: usize,
    pub tail: usize,
}
impl std::fmt::Debug for BaseRunPair {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{:?}{}{:?}", self.head, self.basis, 
            self.tail, !self.basis)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct RunPair<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    pub head: Run<T>,
    pub tail: Run<T>,
}

impl <T> std::fmt::Debug for RunPair<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.head, self.tail)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Pair<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    pub head: T,
    pub tail: T,
}

impl <T> std::fmt::Debug for Pair<T> 
where T: PartialEq + Eq + std::fmt::Debug + Clone
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.head, self.tail)
    }
}


pub struct RunLengthEncoder;
impl RunLengthEncoder {

    // Scan for repeated elements in the list
    pub fn encode<T: PartialEq + Eq + std::fmt::Debug + Clone>(input: &Vec<T>) 
        -> Vec<Run<T>> 
    {
        let mut runs = Vec::new();
        let mut prev: &T = &input[0];
        let mut cnt = 0;
        for current in input.iter() {
            if current != prev {
                runs.push(Run { len: cnt, data: [prev.clone()].to_vec() });
                cnt = 1;
                prev = current;
            } else {
                cnt += 1;
            }
        }
        runs.push(Run { len: cnt, data: [prev.clone()].to_vec() });
        assert!(runs.len() != 0);
        runs
    }

    fn scan<T: PartialEq + Eq + std::fmt::Debug + Clone>(
        window: &[T], chunk_len: usize
    ) -> usize
    {
        let target = &window[0..chunk_len];
        *&window.chunks(chunk_len).take_while(|e| *e == target).count()
    }



    fn scan_window<T: PartialEq + Eq + std::fmt::Debug + Clone>(
        window: &[T], chunk_len: usize, thresh: usize,
    ) -> Option<Run<T>> 
    {
        let target = &window[0..chunk_len];
        let runlen = &window.chunks(chunk_len).take_while(|e| *e == target)
            .count();
        if *runlen >= thresh {
            return Some(Run { len: *runlen, data: target.to_vec() });
        }
        None
    }

    pub fn encode_block<T: PartialEq + Eq + std::fmt::Debug + Clone>(
        input: &Vec<T>, chunk_len: usize, thresh: usize,
    ) -> Vec<Run<T>> 
    {
        let mut res = Vec::new();
        let mut cur = 0;
        while cur < input.len() {
            let window = &input[cur..];
            if let Some(run) = Self::scan_window(window, chunk_len, thresh) {
                cur += run.len * run.data.len();
                res.push(run);
            } else { 
                cur += 1;
            }
        }
        res
    }

    pub fn encode_variable<T: PartialEq + Eq + std::fmt::Debug + Clone>(
        input: &Vec<T>
    ) -> Vec<Run<T>> 
    {
        let mut res = Vec::new();
        let mut cur = 0;
        'top: while cur < (input.len() - 1) {
            let window = &input[cur..];

            for chunk_len in (2..(window.len() / 2)) {
                let res = Self::scan(window, chunk_len);
                if res > 1 {
                    cur += chunk_len * res;
                    continue 'top;
                }
            }


            cur += 1;

        }
        res
    }

    pub fn encode_pairs<T: PartialEq + Eq + std::fmt::Debug + Clone>(
        input: &Vec<T>
    ) -> Vec<Pair<T>> 
    {
        assert!(input.len() > 1);

        // If there are an odd number of runs, just ignore the last run.
        let iter = if (input.len() % 2) != 0 {
            &input[0..input.len()-1]
        } else {
            &input
        };

        let pairs = iter.chunks(2).map(|pair| 
            Pair { head: pair[0].clone(), tail: pair[1].clone()
        }).collect_vec();

        assert!(pairs.len() != 0);
        pairs

    }




}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1], "");
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    // Run through the trace and gather outcomes for all conditional branches
    let mut stat = TraceStats::new();
    for record in trace_records.iter().filter(|r| r.is_conditional()) {
        let entry = stat.get_mut(record.pc);
        entry.outcomes.push(record.outcome);
    }
    println!("[*] Found {} conditional branches", stat.num_unique_branches());


    // Iterate over each unique branch
    for (pc, brn) in stat.data.iter().sorted_by(|x, y| {
        x.1.outcomes.len().partial_cmp(&y.1.outcomes.len()).unwrap()}).rev()
    {

        // Get the set of all outcomes for this branch
        let mut outcomes: Vec<Outcome> = Vec::new();
        for bit in brn.outcomes.as_bitvec() {
            outcomes.push((*bit).into());
        }

        let runs = RunLengthEncoder::encode(&outcomes);
        if runs.len() == 1 {
            continue;
        }

        let pairs = RunLengthEncoder::encode_pairs(&runs);
        if pairs.len() <= 2 { continue; }

        let foo = RunLengthEncoder::encode_block(&pairs, 1, 2);
        //let foo = RunLengthEncoder::encode_block(&foo, 1, 2);
        if foo.len() < 1 {
            continue;
        }

        //let bar = RunLengthEncoder::encode_variable(&foo);
        println!("Branch: {:016x}, bits={}, runs={}, pairs={}",
            pc, outcomes.len(), runs.len(), pairs.len()
        );

        for e in foo {
            println!("  {:?}", e);
        }
        println!();


    }
}
