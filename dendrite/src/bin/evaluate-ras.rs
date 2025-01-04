use dendrite::*;
use dendrite::stats::*;
use dendrite::predictor::simple;
use std::env;
use std::collections::VecDeque;

/// A simulated return-address stack (RAS) predictor [with unlimited capacity].
pub struct UnboundedRAS {
    /// Storage for return addresses
    stack: VecDeque<usize>,
    /// Running total of mispredictions
    miss: usize,
    /// Maximum observed stack depth
    max_size: usize,
}
impl UnboundedRAS {
    fn new() -> Self { 
        Self { 
            stack: VecDeque::new(),
            miss: 0,
            max_size: 0,
        }
    }

    /// Push a return address onto the stack
    fn push(&mut self, return_addr: usize) {
        self.stack.push_back(return_addr);
        if self.stack.len() > self.max_size { 
            self.max_size = self.stack.len();
        }
    }

    /// Remove the youngest element from the stack
    fn pop(&mut self) -> Option<usize> {
        self.stack.pop_back()
    }

    /// Return the program counter of the youngest element on the stack
    fn predict(&mut self) -> Option<usize> {
        self.stack.back().copied()
    }

    /// Register a misprediction
    fn inc_miss(&mut self) {
        self.miss += 1;
        println!("MISS");
        for (idx, entry) in self.stack.iter().enumerate() {
            println!("{:3}: {:016x}", idx, entry);
        }
    }

}


fn run_test(records: &[BranchRecord]) {
    let mut ras = UnboundedRAS::new();
    let mut idt = 0;

    for record in records.iter().filter(|r| r.is_procedural()) {
        match record.kind() { 

            // When we encounter a CALL, compute the return address and 
            // push it onto the RAS
            BranchKind::DirectCall | 
            BranchKind::IndirectCall => {
                let return_addr = record.pc + record.ilen();
                println!("{:idt$} C {:016x}, push {:016x} ilen={}",
                    "", record.pc, return_addr,record.ilen(),
                    idt = idt
                );
                idt += 1;

                ras.push(return_addr);
            },

            // When we encounter a RET, check the resolved target against 
            // the predicted return address [and remove it from the stack]
            BranchKind::Return => {
                let return_addr = ras.predict().unwrap();
                let miss = (return_addr != record.tgt);
                let sts = if !miss { "OK" } else { "!!" };
                if miss {
                    ras.inc_miss();
                }
                println!("{:idt$} R {:016x}, pop {:016x?} (tgt={:016x?}) {} ilen={}",
                    "",record.pc, return_addr, record.tgt, sts, record.ilen(),
                    idt = idt,
                );
                idt -= 1;
                ras.pop().unwrap();
            },
            _ => unreachable!(),
        }
    }

    println!();
    println!("misses:    {}", ras.miss);
    println!("max depth: {}", ras.max_size);
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
