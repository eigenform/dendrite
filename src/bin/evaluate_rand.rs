use dendrite::*;
use itertools::*;
use std::env;
use std::time::Instant;
use rand::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1]);
    let trace_records = trace.as_slice();
    let mut stat = BranchStats::new();

    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);
    for record in trace_records {
        // Just ignore unconditional branches
        if !matches!(record.kind, BranchKind::DirectBranch) {
            continue;
        }

        // Predict a random outcome
        let prediction: Outcome = rand::random::<bool>().into();
        let hit = prediction == record.outcome;
        if hit {
            stat.global_hits += 1;
        }
        stat.global_brns += 1;
    }

    println!("[*] Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
        stat.global_hits(), 
        stat.global_brns(), 
        stat.hit_rate() * 100.0, 
        stat.global_miss()
    );


}
