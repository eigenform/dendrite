/// Evaluate a [`SimplePHT`] against one or more traces. 

use dendrite::*;
use dendrite::stats::*;
use dendrite::predictor::pht::*;

use std::env;

fn index_direct(pht: &SimplePHT, pc: usize) -> usize { 
    pc
}

fn test_pht(pht_size: usize, records: &[BranchRecord]) -> BranchStats {
    let mut stat = BranchStats::new();
    let mut pht = SimplePHT::new(
        pht_size, 
        index_direct, 
        SaturatingCounterConfig { 
            max_t_state: 4,
            max_n_state: 4,
            default_state: Outcome::N,
        },
    );

    for record in records.iter().filter(|r| r.is_conditional()) {

        // Use the program counter to get a PHT entry
        let pht_idx = pht.get_index(record.pc);
        let pht_entry = pht.get_entry_mut(pht_idx);

        // Make a prediction
        let prediction = pht_entry.predict();
        let hit = prediction == record.outcome;

        // Update global statistics
        stat.global_brns += 1;
        if hit {
            stat.global_hits += 1;
        }

        // Update per-branch statistics
        let brn_stat = stat.get_mut(record.pc);
        brn_stat.outcomes.push(record.outcome);
        brn_stat.occ += 1;
        if hit { 
            brn_stat.hits += 1;
        }

        // Update the PHT entry according to the outcome
        pht_entry.update(record.outcome);

    }

    stat
}



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace files>", args[0]);
        return;
    }

    let traces = BinaryTraceSet::new_from_slice(&args[1..]);
    for trace in traces {
        println!("[*] {}, {} records", trace.name(), trace.num_entries());

        let stat = test_pht(4096, trace.as_slice());
        println!("Unique branches: {}", stat.num_unique_branches());
        println!("PHT entries: {}", 1 << 12);
        println!("Global hit rate: {:.2}% ({})", 
            stat.hit_rate() * 100.0, 
            stat.global_brns(),
        );

        println!("Low hit-rate branches:");
        for (pc, data) in stat.get_low_rate_branches(4) {
            println!("  {:016x} {:8}/{:8} {:.4}", 
                pc, data.hits, data.occ, data.hit_rate()
            );
        }
        println!("  ...");
        println!();

    }

}
