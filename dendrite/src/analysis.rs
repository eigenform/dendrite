//! Tools for analyzing traces and branch behavior. 

pub mod stats;
pub mod runlength;
pub mod classify;

pub use stats::*;
pub use runlength::*;
pub use classify::*;

use bitvec::prelude::*;
use crate::Outcome;
use std::collections::*;
use itertools::*;

/// A bitvector storing a set of branch outcomes. 
pub struct BranchOutcomes {
    pub data: BitVec,
}
impl BranchOutcomes {
    pub fn new() -> Self {
        Self { data: BitVec::new() }
    }

    pub fn len(&self) -> usize { 
        self.data.len()
    }

    /// Add an outcome to the list.
    pub fn push(&mut self, outcome: impl Into<bool>) {
        self.data.push(outcome.into());
    }

    /// Return a reference to the underlying [BitVec].
    pub fn as_bitvec(&self) -> &BitVec { 
        &self.data 
    }

    pub fn as_bitslice(&self) -> &BitSlice {
        &self.data
    }

    /// Return a mutable reference to the underlying [BitVec].
    pub fn bitvec_mut(&mut self) -> &mut BitVec { 
        &mut self.data 
    }

}

impl BranchOutcomes {
    /// Returns `Some(outcome)` when all outcomes are the same.
    pub fn is_static(&self) -> Option<Outcome> { 
        if self.data.iter().all(|e| e == self.data[0]) {
            Some(self.data[0].into())
        } else { 
            None
        }
    }

    /// Returns true when all outcomes are "taken".
    pub fn is_always_taken(&self) -> bool {
        self.data.count_ones() == self.data.len()
    }

    /// Returns true when all outcomes are "not-taken".
    pub fn is_never_taken(&self) -> bool {
        self.data.count_zeros() == self.data.len()
    }

    /// Returns the number of "taken" outcomes.
    pub fn num_taken(&self) -> usize { 
        self.data.count_ones()
    }

    /// Returns the number of "not-taken" outcomes.
    pub fn num_not_taken(&self) -> usize { 
        self.data.count_zeros()
    }

    /// Return the ratio between taken and not-taken outcomes.
    pub fn taken_ratio(&self) -> f64 { 
        self.num_taken() as f64 / self.data.len() as f64
    }

    /// Return the ratio between not-taken and taken outcomes.
    pub fn not_taken_ratio(&self) -> f64 { 
        self.num_not_taken() as f64 / self.data.len() as f64
    }

    pub fn into_outcomes(&self) -> Vec<Outcome> {
        Outcome::vec_from_bitvec(&self.data)
    }
    pub fn into_runs(&self) -> Vec<Run<Outcome>> {
        Run::new_vec(&self.into_outcomes())
    }
    pub fn into_pairs(&self) -> Vec<RunPair<Outcome>> {
        RunPair::vec_from_runs(&self.into_runs())
    }

    /// Try to find the shortest pattern that perfectly divides the 
    /// entire set of outcomes. 
    pub fn has_uniform_pattern(&self) -> Option<Vec<Outcome>> {
        let window = &self.data;
        for pattern_len in (1..window.len() / 2) {
            if (window.len() % pattern_len) != 0 {
                continue;
            }
            let head = &window[0..pattern_len];
            let mut iter = window.chunks(pattern_len);
            if iter.all(|e| *e == head) {
                let pattern: Vec<Outcome> = head.iter()
                    .map(|x| Into::<Outcome>::into(*x))
                    .collect();
                return Some(pattern);
            }
        }
        None
    }

    pub fn has_uniform_pattern_prefixed(&self) -> Option<Vec<Outcome>> {
        let window = &self.data[1..];

        for pattern_len in (1..window.len() / 2) {
            if (window.len() % pattern_len) != 0 {
                continue;
            }
            let head = &window[0..pattern_len];
            let mut iter = window.chunks(pattern_len);
            if iter.all(|e| *e == head) {
                let pattern: Vec<Outcome> = head.iter()
                    .map(|x| Into::<Outcome>::into(*x))
                    .collect();
                return Some(pattern);
            }
        }
        None
    }



}

/// Simple container for per-branch statistics.
pub struct BranchData {
    /// Number of times this branch was encountered.
    pub occ: usize,

    /// Number of correct predictions for this branch.
    pub hits: usize,

    /// Record of all observed outcomes for this branch.
    pub outcomes: BranchOutcomes,
}

impl BranchData {
    pub fn new() -> Self {
        Self {
            occ: 0,
            hits: 0,
            outcomes: BranchOutcomes::new(),
        }
    }

    /// Return the hit rate for this branch.
    pub fn hit_rate(&self) -> f64 {
        self.hits as f64 / self.occ as f64
    }

    fn has_uniform_runs(pairs: &[RunPair<Outcome>]) 
        -> Option<Vec<RunPair<Outcome>>>
    {
        let window = &pairs;
        for pattern_len in (1..window.len() / 2) {
            if (window.len() % pattern_len) != 0 {
                continue;
            }
            let head = &window[0..pattern_len];
            let mut iter = window.chunks(pattern_len);
            if iter.all(|e| e == head) {
                let pattern = head.to_vec();
                return Some(pattern);
            }
        }
        None
    }

    /// Classify this branch using the observed outcomes. 
    pub fn classify(&self) -> BranchClass {
        // The outcome is always the same
        if let Some(outcome) = self.outcomes.is_static() {
            return BranchClass::Static(outcome);
        }

        let pairs = self.outcomes.into_pairs();
        if let Some(pattern) = Self::has_uniform_runs(&pairs) {
            return BranchClass::UniformPattern(pattern);
        }


        //if let Some(pattern) = self.outcomes.has_uniform_pattern() {
        //    return BranchClass::UniformPattern(pattern);
        //}

        //if let Some(pattern) = self.outcomes.has_uniform_pattern_prefixed() {
        //    return BranchClass::UniformPatternPrefixed(1, pattern);
        //}


        BranchClass::Unknown
    }

}



//impl BranchOutcomes {
//
//    fn find_shortest_global_pattern(data: &[RunPair]) 
//        -> Option<(usize, Vec<RunPair>)>
//    {
//        // Starting from patterns of length one
//        for chunk_len in (1..=(data.len() / 2)) {
//            // Ignore patterns that don't perfectly divide the input
//            if (data.len() % chunk_len) != 0 { 
//                //println!("Cant fit chunk {} into data {}", chunk_len, data.len());
//                continue; 
//            }
//
//            let target = &data[0..chunk_len];
//
//            // How many times does the pattern repeat? 
//            let count = data.chunks(chunk_len)
//                .take_while(|e| *e == target)
//                .count();
//            //println!("Pattern count={}, {:?}", count, target);
//
//            // Only return if this pattern covers the whole input
//            if count == data.len() / chunk_len {
//                let res = data[0..chunk_len].to_vec();
//                return Some((count, res));
//            }
//        }
//        None
//    }
//
//    /// Try to characterize the behavior of this branch based on *local* 
//    /// information (only contained within this set of outcomes). 
//    pub fn characterize(&self) -> BranchClass {
//
//        // All outcomes are the same
//        if let Some(outcome) = self.is_uniform() {
//            return BranchClass::Static(StaticBehavior::UniformOutcome(outcome));
//        }
//
//        // Compress into a series of runs, then pairs of runs
//        let runs = Run::from_bitvec(&self.data);
//        let pairs = RunPair::from_runs(&runs);
//
//        // A single pair of runs is sufficient
//        if pairs.len() == 1 {
//            return BranchClass::Static(StaticBehavior::UniformPair(pairs[0]));
//        }
//
//        // Evenly split into repeating chunks of some length
//        if let Some((cnt, vec)) = Self::find_shortest_global_pattern(&pairs) {
//            //println!("found pattern {:?}", vec);
//            if vec.len() == 1 {
//                return BranchClass::Static(StaticBehavior::UniformPair(vec[0]));
//            } else {
//                return BranchClass::Static(StaticBehavior::UniformPairs(vec));
//            }
//        }
//
//        if pairs.iter().all(|e| e.bias() == pairs[0].bias()) {
//            let heads = pairs.iter().map(|e| e.head().len()).collect_vec();
//            let tails = pairs.iter().map(|e| e.tail().len()).collect_vec();
//            match pairs[0].bias() {
//                // All pairs are balanced
//                RunPairBias::None => {},
//
//                RunPairBias::Head(o) => {
//                    let uniform = tails.iter().all(|e| *e == tails[0]);
//                    if uniform {
//                        return BranchClass::Static(
//                            StaticBehavior::UniformBias(
//                                BiasBehavior::FixedOpposite(tails[0], o, heads)
//                            )
//                        );
//                    }
//                },
//                RunPairBias::Tail(o) => {
//                    let uniform = heads.iter().all(|e| *e == heads[0]);
//                    if uniform {
//                        return BranchClass::Static(
//                            StaticBehavior::UniformBias(
//                                BiasBehavior::FixedOpposite(heads[0], o, tails)
//                            )
//                        );
//                    }
//                },
//            }
//        }
//
//        //let mut tmp = GenericRun::from_base_vec(&pairs);
//        //let x = GenericRun::refine(&tmp);
//        //for e in x {
//        //    println!("{:?}", e);
//        //}
//
//
//        BranchClass::Unknown
//    }
//}
//
//
//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub enum BranchClass {
//    Unknown,
//    UnknownPairs(Vec<RunPair>),
//    Static(StaticBehavior),
//    Dynamic(DynamicBehavior),
//}
//
//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub enum BiasBehavior {
//    Unknown,
//    FixedOpposite(usize, Outcome, Vec<usize>),
//}
//
//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub enum StaticBehavior {
//    Unknown,
//
//    /// Repeating outcome
//    UniformOutcome(Outcome),
//
//    UniformPair(RunPair),
//
//    /// Repeating pairs of runs
//    UniformPairs(Vec<RunPair>),
//
//    UniformBias(BiasBehavior),
//
//    DeepPattern(Vec<Self>),
//
//}
//
//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub enum DynamicBehavior {
//    Unknown,
//}

///// For classifying different patterns of conditional branch outcomes. 
//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//pub enum BranchBehavior {
//    /// A branch with an unknown/uncomputed pattern.
//    Unknown,
//
//    /// A branch with only a single observed outcome.
//    Once(Outcome),
//
//    /// A branch [observed more than once] whose outcome is always the same. 
//    Fixed(usize, Outcome),
//
//    /// A branch whose outcome changes only once after a single run.
//    FlipOnce(Run),
//
//    /// A branch can be described with an arbitrary repeating pattern of runs.
//    Periodic(usize, Vec<Run>),
//
//    /// A branch whose runs are always longer in a particular direction, 
//    /// and where runs of the opposite outcome may be arbitrarily long.
//    Biased(Outcome),
//
//    /// A branch whose runs are always longer in a particular direction, 
//    /// but where runs of the opposite outcome are always the same length.
//    BiasedFixed(Outcome, usize),
//
//}
//impl BranchBehavior {
//    /// Try to compress/characterize this bitvector.
//    pub fn from_bitvec(bits: &BitVec) -> Self { 
//
//        // Trivial cases
//        if bits.len() == 1 {
//            return BranchBehavior::Once(bits[0].into());
//        }
//        if bits.count_ones() == bits.len() {
//            return BranchBehavior::Fixed(bits.len(), Outcome::T);
//        }
//        if bits.count_zeros() == bits.len() {
//            return BranchBehavior::Fixed(bits.len(), Outcome::N);
//        }
//
//        // Run-length description
//        let rle = RunLengthOutcomes::from_bitvec(bits);
//        if let Some(run) = rle.is_bimodal() {
//            return BranchBehavior::FlipOnce(run);
//        }
//        if let Some((pattern_len, pattern)) = rle.is_periodic() {
//            return BranchBehavior::Periodic(pattern_len, pattern);
//        }
//        if let Some((o, len)) = rle.is_biased_fixed() {
//            return BranchBehavior::BiasedFixed(o, len);
//        }
//        if let Some(o) = rle.is_biased() {
//            return BranchBehavior::Biased(o);
//        }
//
//        BranchBehavior::Unknown
//    }
//}


