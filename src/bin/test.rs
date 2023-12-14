
use dendrite::*;
use bitvec::prelude::*;
use dendrite::tage::*;


fn main() {
    let mut ghr  = GlobalHistoryRegister::new(64);
    //ghr.randomize();

    let get_base_pc = |x: usize| { x & 0xfff };
    let mut tage = TAGEPredictor::new(TAGEBaseComponent::new(
        SaturatingCounter::new(2, 2, Outcome::N),
        1024, get_base_pc
    ));

    let foo = |x: usize| { x & 0xfff };
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(2, 2, Outcome::N)), 1024, 
        0..=15, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(2, 2, Outcome::N)), 1024, 
        0..=31, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(2, 2, Outcome::N)), 1024, 
        0..=63, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(2, 2, Outcome::N)), 1024, 
        0..=7, 12, foo
    ));


    let pattern = [
        Outcome::N, Outcome::N, Outcome::N, Outcome::T,
    ];
    let pc = 0x1000_0000;
    for i in 0..64 {
        // Make a prediction
        println!("[*] Branch @ {:016x}", pc);
        println!("  ghr={}", ghr);
        let p = tage.predict(pc);
        println!("  pred={:?}", p);

        // The outcome is resolved
        let outcome = pattern[i % pattern.len()];
        if outcome != p.outcome {
            println!("  misprediction");
            println!();
        }

        // Update the predictor based on the outcome. 
        // The state of global history has not changed. 
        tage.update(pc, p, outcome);

        // Update the state of global history. 
        ghr.shift_by(1);
        ghr.data_mut().set(0, outcome.into());

        // Update the predictor's folded view of global history 
        tage.update_history(&ghr);
    }


}


