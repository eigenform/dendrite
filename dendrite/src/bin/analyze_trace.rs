
use dendrite::*;
use itertools::*;
use std::env;
use std::collections::*;
use bitvec::prelude::*;

/// For classifying different patterns of conditional branch outcomes. 
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum BranchPattern {
    /// A branch with an unknown/uncomputed pattern.
    Unknown,

    /// A branch whose outcome is always the same. 
    Fixed(Outcome),

    /// A branch whose outcome changes only once.
    FlipOnce,

    /// A branch can be described with a repeating pattern of runs.
    GlobalPattern(usize, Vec<(Outcome, usize)>),

    /// A branch is biased towards some outcome, but the history is punctuated 
    /// [at potentially many arbitrary points] by fixed-length runs of the 
    /// opposite outcome. 
    BiasedWithPerturbations(Outcome),

}
impl BranchPattern {
    pub fn from_bitvec(bits: &BitVec) -> Self { 
        let initial_outcome = bits[0];

        // The branch is always taken
        if bits.count_ones() == bits.len() {
            return BranchPattern::Fixed(Outcome::T);
        }
       
        // The branch is never taken
        if bits.count_zeros() == bits.len() {
            return BranchPattern::Fixed(Outcome::N);
        }

        // Capture a list of runs (of repeated outcomes)
        let mut rle = Vec::new();
        let mut cnt = 0;
        let mut b = bits[0];
        for bit in bits.iter() {
            if bit == !b {
                rle.push(cnt);
                cnt = 1;
                b = *bit;
            } else {
                cnt += 1;
            }
        }
        rle.push(cnt);

        // The outcome only flips once
        if rle.len() == 2 {
            return BranchPattern::FlipOnce;
        }

       // Obviously biased with fixed-length perturbations
        let mut iter = if rle.len() % 2 == 0 {
            rle.chunks(2)
        } else {
            rle[0..rle.len()-1].chunks(2)
        };
        let first = iter.next().unwrap();
        let o: Outcome = initial_outcome.into();

        if iter.all(|e| e[0] == first[0]) {
            return BranchPattern::BiasedWithPerturbations(!o);
        }
        if iter.all(|e| e[1] == first[1]) {
            return BranchPattern::BiasedWithPerturbations(o);
        }


        // A repeating pattern of runs. 
        for pattern_len in 2..=(bits.len() / 2) {
            let mut iter = rle.chunks(pattern_len);
            let num_iters = iter.len();
            let first = iter.next().unwrap();
            if iter.len() == 0 {
                continue; 
            }
            if iter.all(|e| e == first) {
                let initial: Outcome = initial_outcome.into();
                let mut pattern = Vec::new();
                let mut o = initial;
                for run in first { 
                    pattern.push((o, *run));
                    o = !o;
                }
                return BranchPattern::GlobalPattern(num_iters, pattern);
            }
        }

        println!("{:?}", rle);
        let iter = rle.chunks(2);
        for c in iter {
            println!("{:?}", c);
        }

        //println!("{:?}", rle);
        BranchPattern::Unknown

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

    let mut stat = BranchStats::new();
    for record in trace_records.iter().filter(|r| r.is_conditional()) {
        let entry = stat.get_mut(record.pc);
        entry.pat.push(record.outcome.into());
    }

    println!("[*] Found {} unique branches", stat.num_unique_branches());
    let mut pats = HashMap::new();

    for (pc, brn) in stat.data.iter().sorted_by(|x, y| {
        x.1.pat.len().partial_cmp(&y.1.pat.len()).unwrap()
    }).rev()
    {
        // Ignore branches that are encountered only once
        if brn.pat.len() == 1 { 
            continue; 
        }

        let pat = BranchPattern::from_bitvec(&brn.pat);
        let e = pats.entry(pat.clone()).or_insert(0);
        *e += 1;

        if !matches!(pat, BranchPattern::Unknown) {
            continue;
        }

        println!("[*] Address: {:016x}, len={}", pc, brn.pat.len());
        println!("Ratio: t={}, n={}", brn.pat.count_ones(), brn.pat.count_zeros()); 
        let patfmt = if brn.pat.len() > 64 { &brn.pat[0..64] } else { &brn.pat };
        println!("Outcomes: {:b}", patfmt);
        println!("Pattern: {:?}", pat);
        println!();

    }

    let iter = pats.iter().sorted_by(|x, y| x.1.partial_cmp(y.1).unwrap()).rev();
    for (pattern, cnt) in iter { 
        println!("occ={:8} {:?}", cnt, pattern);
    }



}
