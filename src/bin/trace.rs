
use dendrite::*;

fn main() {

    let mut e = TraceEmitter::new(0x1000_0000);
    let lab = e.create_label();
    let start = e.create_label();
    e.bind_label(start);
    e.branch_to_label(lab, BranchPattern::NeverTaken);
    e.branch_to_label(lab, BranchPattern::NeverTaken);
    e.branch_to_label(lab, BranchPattern::TakenPeriodic(4));
    e.bind_label(lab);
    e.jump_to_label(start);



    let records = e.simulate_for(64);
    for r in records.iter() {
        println!("{:08x?}", r);
    }
}
