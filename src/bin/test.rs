
use dendrite::*;
use dendrite::predictor::tage::*;
use bitvec::prelude::*;

fn main() {
    let mut ghr  = GlobalHistoryRegister::new(64);

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

    // Randomize the state of global history before we start evaluating 
    for _ in 0..64 {
        ghr.shift_by(1);
        ghr.data_mut().set(0, rand::random());
        tage.update_history(&ghr);
    }

    // NOTE: Temporary, just making sure things are vaguely working~
    for i in 0..8 {
        println!("Generation {}", i);
        let x = test_pattern(&mut ghr, &mut tage, 
            0x1000_0444, 
            &[ Outcome::N, Outcome::N, Outcome::N, Outcome::T],
            1024,
        );
        println!("  {:?} {}", 1.0 - (x as f64 / 1024.0), x );

        let x = test_pattern(&mut ghr, &mut tage, 
            0x1000_0162, 
            &[ Outcome::T ],
            4,
        );
        println!("  {:?} {}", 1.0 - (x as f64 / 1024.0), x );

        let x = test_pattern(&mut ghr, &mut tage, 
            0x1000_0f5a, 
            &[ Outcome::T, Outcome::T, Outcome::N, Outcome::T ],
            512,
        );
        println!("  {:?} {}", 1.0 - (x as f64 / 1024.0), x );

        let x = test_pattern(&mut ghr, &mut tage, 
            0x1000_0f5d, 
            &[ Outcome::N],
            4,
        );
        println!("  {:?} {}", 1.0 - (x as f64 / 1024.0), x );
    }

}

fn test_pattern(
    ghr: &mut GlobalHistoryRegister, 
    tage: &mut TAGEPredictor,
    pc: usize,
    pattern: &[Outcome],
    iters: usize,
) -> usize
{
    let mut misses = 0;

    for i in 0..iters {
        // Make a prediction
        let p = tage.predict(pc);

        // Resolve the branch outcome
        let outcome = pattern[i % pattern.len()];
        if outcome != p.outcome {
            misses += 1;
        }

        // Update the predictor based on the outcome. 
        tage.update(pc, p, outcome);

        // Update the state of global history to reflect the resolved outcome
        ghr.shift_by(1);
        ghr.data_mut().set(0, outcome.into());

        // Update the predictor's [folded] view of global history 
        tage.update_history(&ghr);
    }
    misses
}

