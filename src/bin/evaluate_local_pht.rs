use dendrite::*;
use itertools::*;
use std::env;
use std::time::Instant;
use rand::prelude::*;

/// A table of saturating counters. 
pub struct PatternHistoryTable { 
    data: Vec<SaturatingCounter>,
    size: usize,
}
impl PatternHistoryTable {
    pub fn new(size: usize) -> Self { 
        let cfg = SaturatingCounterConfig { 
            default_state: Outcome::N,
            max_n_state: 1,
            max_t_state: 1,
        };

        let data = vec![cfg.build(); size];
        Self { 
            data,
            size,
        }
    }
}
impl PredictorTable for PatternHistoryTable {
    type Input<'a> = usize;
    type Index = usize;
    type Entry = SaturatingCounter;

    fn size(&self) -> usize { self.size }

    // Directly indexed with low bits from the program counter
    fn get_index(&self, pc: usize) -> usize { 
        pc & self.index_mask()
    }

    fn get_entry(&self, idx: usize) -> &SaturatingCounter { 
        let index = idx & self.index_mask();
        &self.data[index]
    }
    fn get_entry_mut(&mut self, idx: usize) -> &mut SaturatingCounter { 
        let index = idx & self.index_mask();
        &mut self.data[index]
    }

}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1]);
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    for pht_size in 1..=15 {
        let num_entries = (1 << pht_size);
        let mut stat = BranchStats::new();
        let mut pht = PatternHistoryTable::new(num_entries);

        for record in trace_records {
            // Ignore unconditional branches
            if !matches!(record.kind, BranchKind::DirectBranch) {
                continue;
            }
            stat.global_brns += 1;

            // Use the program counter to get a PHT entry
            let pht_idx = pht.get_index(record.pc);
            let pht_entry = pht.get_entry_mut(pht_idx);

            // Make a prediction
            let prediction = pht_entry.predict();
            let hit = prediction == record.outcome;
            if hit {
                stat.global_hits += 1;
            }

            // Update the PHT entry according to the outcome
            pht_entry.update(record.outcome);

            // Update per-branch statistics
            let brn_stat = stat.get_mut(record.pc);
            brn_stat.pat.push(record.outcome.into());
            brn_stat.occ += 1;
            if hit { 
                brn_stat.hits += 1;
            }

        }

        println!("[*] Unique branches: {}", stat.num_unique_branches());
        println!("[*] PHT entries: {}", num_entries);
        println!("     Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
            stat.global_hits(), 
            stat.global_brns(), 
            stat.hit_rate() * 100.0, 
            stat.global_miss()
        );
        let low_rate_iter = stat.data.iter()
            .filter(|(_, s)| s.occ > 100 && s.hit_rate() <= 0.55)
            //.sorted_by(|x,y| { x.1.hit_rate().partial_cmp(&y.1.hit_rate()).unwrap() })
            .sorted_by(|x,y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() }).rev()
            .take(32);
        for (pc, s) in low_rate_iter {
            let pat = if s.pat.len() > 64 {
                let slice = &s.pat.as_bitslice()[s.pat.len()-64..s.pat.len()];
                format!("{:b}", slice)
            } else {
                format!("{:b}", s.pat)
            };
            println!("    {:016x}: {:6}/{:6} ({:.4}) H={:.2} {}",
                pc, s.hits, s.occ, s.hit_rate(), s.shannon_entropy(), pat);
        }



    }


}
