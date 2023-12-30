
use dendrite::*;
use dendrite::stats::*;
use dendrite::predictor::simple;
use std::env;

fn run_test(records: &[BranchRecord], p: impl SimplePredictor) {
    let mut stat = BranchStats::new();

    for record in records.iter().filter(|r| r.is_conditional()) { 
        stat.update_global(record, p.predict());
    }

    println!("[*] {:20} Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
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
        println!("usage: {} <trace file>", args[0]);
        return;
    }
    let trace = BinaryTrace::from_file(&args[1]);
    let trace_records = trace.as_slice();
    run_test(trace_records, simple::RandomPredictor);
    run_test(trace_records, simple::TakenPredictor);
    run_test(trace_records, simple::NotTakenPredictor);

}
