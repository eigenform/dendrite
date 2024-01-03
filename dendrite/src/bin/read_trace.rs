
use dendrite::*;
use itertools::*;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1], "");
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);
    for record in trace_records {
        println!("{:016x} {:016x} {:?} {:?}", 
            record.pc, record.tgt, record.outcome, record.kind);
    }


}
