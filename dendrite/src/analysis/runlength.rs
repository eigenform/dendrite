
use crate::Outcome;
use bitvec::prelude::*;
use bitvec::ptr::write;
use itertools::*;
use std::collections::*;
use std::fmt::Debug;

/// Describes a repeating sequence of some type T. 
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Run<T> 
where T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
{
    /// The number of times this sequence is repeated. 
    cnt: usize,
    /// The repeating sequence. 
    data: Vec<T>,
}

impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash> Run<T> {
    pub fn new(cnt: usize, data: Vec<T>) -> Self { 
        assert!(cnt != 0);
        Self { cnt, data }
    }

    pub fn len(&self) -> usize { 
        self.data.len()
    }

    pub fn total_len(&self) -> usize { 
        self.len() * self.cnt
    }

    pub fn new_vec(input: &[T]) -> Vec<Self> {
        let mut res = Vec::new();
        let mut cur = &input[0];
        let mut cnt = 0;
        for x in input {
            if x != cur {
                res.push(Self::new(cnt, [cur.clone()].to_vec()));
                cur = &x;
                cnt = 1;
            } 
            else {
                cnt += 1;
            }
        }
        res.push(Self::new(cnt, [cur.clone()].to_vec()));

        res
    }
}

impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash>
std::fmt::Debug for Run<T> 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        if self.data.len() < 2 {
            write!(f, "{}{:?}", self.cnt, self.data[0])
        } else {
            write!(f, "{}{:?}", self.cnt, self.data)
        }
    }
}

/// A pair of [`Run<T>`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RunPair<T> 
where T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
{
    head: Run<T>,
    tail: Run<T>,
}

impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash>
RunPair<T> 
{
    pub fn new(head: Run<T>, tail: Run<T>) -> Self { 
        Self { head, tail }
    }

    pub fn vec_from_runs(input: &[Run<T>]) -> Vec<Self> {
        let mut res = Vec::new();
        for pair in input.chunks_exact(2) {
            res.push(RunPair::new(pair[0].clone(), pair[1].clone()));
        }
        res
    }

}

impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash>
std::fmt::Debug for RunPair<T> 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{:?}{:?}", self.head, self.tail)
    }
}



//pub struct RunLengthAnalysis {
//    runs: Vec<RunPair>,
//}
//
///// Representing a sequence of repeated branch outcomes. 
//#[derive(Copy, Clone, PartialEq, Eq, Hash)]
//pub struct Run {
//    /// The number of times this outcome was repeated
//    pub len: usize,
//    /// The branch outcome
//    pub outcome: Outcome,
//}
//impl Run {
//    pub fn new(outcome: Outcome, len: usize) -> Self {
//        assert!(len != 0);
//        Self { outcome, len }
//    }
//    pub fn len(&self) -> usize { self.len }
//    pub fn outcome(&self) -> Outcome { self.outcome }
//
//    /// Given a [BitVec], create a list of [Run]. 
//    pub fn from_bitvec(bits: &BitVec) -> Vec<Self> {
//        let mut runs = Vec::new();
//        let mut prev: Outcome = bits[0].into();
//        let mut cnt = 0;
//        for bit in bits.iter() {
//            let outcome: Outcome = (*bit).into();
//            if outcome != prev {
//                runs.push(Run::new(prev, cnt));
//                cnt = 1;
//                prev = outcome;
//            } else { 
//                cnt += 1;
//            }
//        }
//        runs.push(Run::new(prev, cnt));
//        runs
//    }
//}
//impl std::fmt::Display for Run {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        let o = match self.outcome { 
//            Outcome::T => "t",
//            Outcome::N => "n",
//        };
//        write!(f, "{}{}", self.len, o)
//    }
//}
//impl std::fmt::Debug for Run {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "{}", self)
//    }
//}
//
//
//#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
//pub enum RunPairBias {
//    None,
//    Head(Outcome),
//    Tail(Outcome),
//}
//
///// Representing a pair of runs.
//#[derive(Copy, Clone, PartialEq, Eq, Hash)]
//pub struct RunPair {
//    pub head: Run,
//    pub tail: Run,
//}
//impl RunPair {
//    pub fn new(head: Run, tail: Run) -> Self { 
//        assert!(head.outcome != tail.outcome);
//        Self { head, tail }
//    }
//
//    /// Given a list of [Run], create a list of [RunPair]. 
//    /// Ignores the final run if the number of runs is not even.
//    pub fn from_runs(runs: &Vec<Run>) -> Vec<Self> {
//        let iter = if (runs.len() % 2) != 0 {
//            &runs[0..runs.len()-1]
//        } else {
//            &runs
//        };
//        iter.chunks(2).map(|pair| RunPair::new(pair[0], pair[1]))
//            .collect_vec()
//    }
//
//
//    pub fn head(&self) -> &Run { &self.head }
//    pub fn tail(&self) -> &Run { &self.tail }
//    pub fn len(&self) -> usize { self.head.len + self.tail.len }
//
//    pub fn bias(&self) -> RunPairBias { 
//        if self.head.len == self.tail.len { 
//            return RunPairBias::None; 
//        }
//        if self.head.len > self.tail.len {
//            RunPairBias::Head(self.head.outcome)
//        } else {
//            RunPairBias::Tail(self.head.outcome)
//        }
//    }
//
//    pub fn ratio(&self, basis: Outcome) -> f64 { 
//        if self.head.outcome == basis { 
//            self.head.len as f64 / self.len() as f64
//        } else {
//            self.tail.len as f64 / self.len() as f64
//        }
//    }
//}
//impl std::fmt::Display for RunPair {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "{}{}", self.head, self.tail)
//    }
//}
//impl std::fmt::Debug for RunPair {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "{:?}{:?}", self.head, self.tail)
//    }
//}
//
//
//pub struct RunLengthPairs {
//    pub data: Vec<RunPair>,
//}
//impl RunLengthPairs {
//    pub fn from_runs(runs: &RunLengthOutcomes) -> Self { 
//        let data: Vec<RunPair> = {
//            runs.pairwise().map(|pair| RunPair::new(pair[0], pair[1]))
//                .collect()
//        };
//
//        Self { data }
//    }
//}
//
//
///// A set of observed branch outcomes in run-length format. 
//#[derive(Debug)]
//pub struct RunLengthOutcomes {
//    pub num_bits: usize,
//    pub data: Vec<Run>,
//}
//impl RunLengthOutcomes {
//    pub fn num_runs(&self) -> usize { self.data.len() }
//    pub fn from_bitvec(bits: &BitVec) -> Self {
//        let mut data = Vec::new();
//        let mut prev_outcome: Outcome = bits[0].into();
//        let mut cnt = 0;
//        for bit in bits.iter() {
//            let outcome: Outcome = (*bit).into();
//            if outcome != prev_outcome {
//                data.push(Run { outcome: prev_outcome, len: cnt });
//                cnt = 1;
//                prev_outcome = outcome;
//            } else { 
//                cnt += 1;
//            }
//        }
//        data.push(Run { outcome: prev_outcome, len: cnt });
//        Self { num_bits: bits.len(), data }
//    }
//
//    /// Iterate over pairs of runs 
//    pub fn pairwise(&self) -> std::slice::Chunks::<'_, Run> {
//        if self.data.len() % 2 == 0 {
//            self.data.chunks(2)
//        } else {
//            self.data[0..self.data.len()-1].chunks(2)
//        }
//    }
//
//    /// Iterate over n-tuples of runs
//    pub fn chunks(&self, n: usize) -> std::slice::Chunks<'_, Run> {
//        let rem = self.data.len() % n; 
//        if rem == 0 { 
//            self.data.chunks(n)
//        } else { 
//            self.data[0..self.data.len()-rem].chunks(n)
//        }
//    }
//
//    pub fn is_bimodal(&self) -> Option<Run> {
//        if self.data.len() == 2 {
//            Some(self.data[0])
//        } else { 
//            None
//        }
//    }
//
//    pub fn is_periodic(&self) -> Option<(usize, Vec<Run>)> {
//        if self.data.len() <= 2 {
//            return None;
//        }
//
//        for pattern_len in 2..=(self.data.len() / 2) {
//            let mut chunks = self.chunks(pattern_len);
//            let num_chunks = chunks.len();
//            let first = chunks.next().unwrap();
//            if chunks.len() == 0 {
//                continue; 
//            }
//            if chunks.all(|e| e == first) {
//                let pattern = first.to_vec();
//                return Some((num_chunks, pattern));
//            }
//        }
//        None
//    }
//
//    pub fn is_biased(&self) -> Option<Outcome> {
//        let mut pairs = self.pairwise();
//        if pairs.all(|e| e[0].len > e[1].len) {
//            return Some(self.data[0].outcome);
//        }
//        let mut pairs = self.pairwise();
//        if pairs.all(|e| e[1].len > e[0].len) {
//            return Some(self.data[1].outcome);
//        }
//        None
//    }
//
//
//    pub fn is_biased_fixed(&self) -> Option<(Outcome, usize)> {
//        if self.data.len() <= 2 {
//            return None;
//        }
//
//        // The first run is always fixed, and the second run is always longer.
//        // The branch outcome is biased towards the second run. 
//        let mut pairs = self.pairwise();
//        let first_pair = pairs.next().unwrap();
//        let eq_0 = pairs.all(|pair| {
//            (pair[0] == first_pair[0]) && (pair[1].len > pair[0].len)
//        });
//        if eq_0 {
//            return Some((first_pair[1].outcome, first_pair[0].len));
//        }
//
//        // The second run is always fixed, and the first run is always longer
//        // The branch outcome is biased towards the first run. 
//        let mut pairs = self.pairwise();
//        let first_pair = pairs.next().unwrap();
//        let eq_1 = pairs.all(|pair| {
//            pair[1] == first_pair[1] && (pair[0].len > pair[1].len)
//        });
//        if eq_1 {
//            return Some((first_pair[0].outcome, first_pair[1].len));
//        }
//
//        None
//    }
//}


//pub struct PatternSpec { 
//    chunk_len: usize,
//    repeat: usize,
//}
//
///// For describing nested runs of some base datatype T. 
//#[derive(Clone, PartialEq, Eq, Hash)]
//pub enum GenericRun<T> 
//where T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
//{
//    /// An instance of the base datatype T
//    Base { data: T },
//    /// A single repeated element
//    Run { len: usize, data: Box<Self> },
//    /// An arbitrary number of repeated elements
//    Pattern { len: usize, data: Vec<Self> },
//}
//impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash>
//std::fmt::Debug for GenericRun<T> 
//{
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        match self { 
//            Self::Base { data } => {
//                write!(f, "{:?}", data)
//            },
//            Self::Run { len, data } => {
//                write!(f, "{}{:?}", len, data)
//            },
//            Self::Pattern { len, data } => {
//                write!(f, "{}{:?}", len, data)
//            },
//        }
//    }
//}
//
//
//impl <T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash>
//GenericRun<T> 
//{
//    /// Convert a [Vec<T>] into a [Vec<GenericRun<T>>].
//    pub fn from_base_vec(vec: &Vec<T>) -> Vec<Self> {
//        vec.iter().map(|e| Self::Base { data: e.clone() })
//            .collect_vec()
//    }
//
//    fn find_pattern(input: &[Self])
//        -> Option<PatternSpec> 
//    {
//        let map = Self::occurence_map(&input);
//
//        if *map.get(&input[0]).unwrap() == 1 {
//            return None;
//        }
//
//        for chunk_len in (1..(input.len() / 2)) {
//            let target = &input[0..chunk_len];
//
//            // If any of the elements occur only once in the input, this 
//            // chunk cannot be a repeating pattern
//            if target.iter().any(|e| *map.get(e).unwrap() == 1) {
//                continue;
//            }
//
//
//            let count = input.chunks(chunk_len)
//                .take_while(|e| *e == target)
//                .count();
//            if count > 1 {
//                return Some(PatternSpec { chunk_len, repeat: count });
//            }
//        }
//        None
//    }
//
//    fn occurence_map(input: &[Self]) -> HashMap<&Self, usize> {
//        let mut map = HashMap::new();
//        for e in input { 
//            let foo = map.entry(e).or_insert(0);
//            *foo += 1;
//        }
//        map
//    }
//
//    pub fn refine(input: &[Self]) -> Vec<Self> {
//        let mut res = Vec::new();
//        let map = Self::occurence_map(&input);
//        let mut cursor = 0;
//
//        while cursor < input.len() {
//            if *map.get(&input[cursor]).unwrap() == 1 {
//                res.push(input[0].clone());
//                cursor += 1;
//                continue;
//            }
//
//            let window = &input[cursor..];
//            if let Some(spec) = Self::find_pattern(window) {
//                if spec.chunk_len == 1 {
//                    res.push(Self::Run { 
//                        len: spec.repeat, 
//                        data: Box::new(window[0].clone())
//                    });
//                    cursor += spec.repeat;
//                } 
//                else if spec.chunk_len > 1 {
//                    let pattern = &window[0..spec.chunk_len];
//                    res.push(Self::Pattern {
//                        len: spec.repeat, 
//                        data: pattern.to_vec(),
//                    });
//                    cursor += spec.chunk_len * spec.repeat;
//                }
//            } 
//            else {
//                res.push(window[0].clone());
//                cursor += 1;
//            }
//        }
//
//        res
//    }
//}


//#[derive(Clone, PartialEq, Eq)]
//pub enum RunIR<T> 
//where T: Clone + Debug + PartialEq + Eq
//{
//    Value(T),
//    Run(usize, Vec<Self>),
//}
//impl <T: Clone + Debug + PartialEq + Eq> RunIR<T> {
//    pub fn new_run(len: usize, data: Vec<Self>) -> Self { 
//        Self::Run(len, data)
//    }
//
//    /// Starting from the beginning of the input, count the number of times 
//    /// that the pattern `pat` is repeated.
//    fn find_runlen(input: &[Self], pat: &[Self])
//        -> Option<usize> 
//    {
//        if input.len() < pat.len() {
//            return None;
//        }
//        let num_chunks = input.len() / pat.len();
//        let mut count = 0;
//        for n in 0..num_chunks { 
//            let start = pat.len() * n;
//            let end = start + pat.len();
//            let tgt = &input[start..end];
//            assert!(tgt.len() == pat.len());
//            if pat != tgt { 
//                break; 
//            }
//            count += 1;
//        }
//        Some(count)
//    }
//
//    /// Starting from the beginning of the input, try to find a repeating 
//    /// pattern.
//    fn find_pattern(input: &[Self]) -> Option<(usize, usize)> {
//        for chunk_len in 1..=(input.len() / 2) {
//            let pat = &input[0..chunk_len];
//            if let Some(count) = Self::find_runlen(input, pat) {
//                return Some((chunk_len, count));
//            }
//        }
//        None
//    }
//
//
//    fn compress_slice(input: &[Self]) -> Vec<Self> {
//        let mut res = Vec::new();
//        let mut cur = 0;
//        while cur < input.len() {
//            let window = &input[cur..];
//            if let Some((pat_len, count)) = Self::find_pattern(window) {
//                cur += (pat_len * count);
//            } else { 
//                cur += 1;
//            }
//        }
//    }
//
//    pub fn encode(&mut self) {
//        match self {
//            Self::Value(_) => {},
//            Self::Run(len, data) => {
//                let mut res = Vec::new();
//            },
//        }
//    }
//
//    pub fn compress(&mut self) -> Option<Self> {
//        match self { 
//            Self::Value(_) => None,
//            Self::Run(len, data) => {
//                None
//            },
//        }
//    }
//
//}
//
//pub struct RunInterpreter<T> 
//where T: Clone + Debug + PartialEq + Eq
//{
//    data: RunIR<T>,
//}
//impl <T: Clone + Debug + PartialEq + Eq> RunInterpreter<T> {
//    pub fn new(input: &[T]) -> Self { 
//        let values = input.iter().map(|e| RunIR::Value(*e)).collect_vec();
//        let data = RunIR::new_run(1, values);
//        Self { data } 
//    }
//
//    pub fn compress(&mut self) -> bool {
//
//        for e in self.data {
//            match e {
//                RunIR::Value(_) => unreachable!(),
//                RunIR::Run(len, data) => {
//                }
//            }
//        }
//    }
//
//}
//
//
//
//#[derive(Clone, PartialEq, Eq)]
//pub struct Run<T>(usize, T) 
//    where T: Clone + Debug + PartialEq + Eq;
//impl <T: Clone + Debug + PartialEq + Eq> Debug for Run<T> 
//{
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}<{:?}>", self.0, self.1)
//    }
//}
//impl <T: Clone + Debug + PartialEq + Eq> Run<T> {
//    pub fn new(len: usize, data: T) -> Self { 
//        Self(len, data)
//    }
//}
//
//
//#[derive(Clone, PartialEq, Eq)]
//pub struct RunList<T> 
//where T: Clone + Debug + PartialEq + Eq
//{
//    len: Vec<usize>,
//    data: Vec<T>,
//}
//
//impl <T: Clone + Debug + PartialEq + Eq> 
//Debug for RunList<T> 
//{
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}<{:?}>", self.len, self.data)
//    }
//}
//
//impl <T: Clone + Debug + PartialEq + Eq> 
//RunList<T> 
//{
//    pub fn new(len: usize, data: Vec<T>) -> Self { 
//        Self { len, data }
//    }
//
//    pub fn new_from_slice(input: &[T]) -> Self { 
//    }
//}









