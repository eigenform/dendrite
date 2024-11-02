use dendrite::*;
use dendrite::stats::*;
use dendrite::predictor::simple;
use std::env;
use std::collections::VecDeque;

pub struct SimpleRAS {
    stack: VecDeque<usize>,
}
impl SimpleRAS {
    fn push(&mut self, return_addr: usize) {
        self.stack.push_back(return_addr);
    }

    fn predict(&mut self) -> Option<usize> {
        if let Some(tgt) = self.stack.pop_back() {
            Some(tgt)
        } else { 
            None
        }
    }
}


// FIXME: I think you'd need to add bits in the record for capturing the 
// instruction length ... argh x86 
fn run_test(records: &[BranchRecord]) {
    let mut stat = TraceStats::new();

    for record in records.iter().filter(|r| r.is_procedural()) {
        match record.kind { 
            BranchKind::DirectCall | 
            BranchKind::IndirectCall => {
            },
            _ => unreachable!(),
        }
        println!("{:016x?}", record);
        //stat.update_global(record, p.predict());
    }

    //println!("  {:20} Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
    //    p.name(),
    //    stat.global_hits(), 
    //    stat.global_brns(), 
    //    stat.hit_rate() * 100.0, 
    //    stat.global_miss()
    //);
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
        run_test(records);
    }

}
