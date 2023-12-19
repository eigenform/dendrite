
use dendrite::*;

fn main() {

    let mut e = TraceAssembler::new(0x1000_0000);
    let lab = e.create_label();
    let start = e.create_label();
    e.bind_label(start);
    e.branch_to_label(lab, BranchPattern::NeverTaken);
    e.branch_to_label(lab, BranchPattern::Pattern(
            &[Outcome::T, Outcome::T, Outcome::N, Outcome::N]
    ));
    e.branch_to_label(lab, BranchPattern::NeverTaken);
    e.branch_to_label(lab, BranchPattern::TakenPeriodic(4));

    e.bind_label(lab);
    e.pad_align(0x0000_1000);
    e.jump_to_label(start);

    let t = e.compile(64);
    for r in t.data.iter() {
        println!("{:08x?}", r);
    }

}
