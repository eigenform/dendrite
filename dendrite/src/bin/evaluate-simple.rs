/// Evaluate a [`SimplePredictor`] against one or more traces. 

use dendrite::*;
use dendrite::stats::*;
use dendrite::predictor::simple;
use std::env;

fn run_test(records: &[BranchRecord], p: impl SimplePredictor) {
    let mut stat = TraceStats::new();

    for record in records.iter().filter(|r| r.is_conditional()) { 
        stat.update_global(record, p.predict());
    }

    println!("  {:20} Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
        p.name(),
        stat.global_hits(), 
        stat.global_brns(), 
        stat.hit_rate() * 100.0, 
        stat.global_miss()
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace files>", args[0]);
        return;
    }
    let traces = BinaryTraceSet::new_from_slice(&args[1..]);

    for trace in traces {
        if trace.num_entries() < 100 { continue; }
        println!("[*] {}", trace.name());
        let records = trace.as_slice();
        run_test(records, simple::RandomPredictor);
        run_test(records, simple::TakenPredictor);
        run_test(records, simple::NotTakenPredictor);
    }

}
