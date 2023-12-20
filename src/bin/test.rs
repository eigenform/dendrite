
use dendrite::*;
use bitvec::prelude::*;

fn main() {

    let get_base_pc = |x: usize| { x & 0xfff };
    let mut tage_cfg = TAGEConfig::new(
        TAGEBaseConfig { 
            ctr: SaturatingCounterConfig {
                max_t_state: 2,
                max_n_state: 2,
                default_state: Outcome::N,
            },
            size: 1024,
            index_fn: get_base_pc,
        },
    );

    for ghr_range_hi in &[7, 15, 31, 63] {
        tage_cfg.add_component(TAGEComponentConfig {
            size: 1024,
            ghr_range: 0..=*ghr_range_hi,
            tag_bits: 12,
            pc_sel_fn: get_base_pc,
            ctr: SaturatingCounterConfig {
                max_t_state: 2,
                max_n_state: 2,
                default_state: Outcome::N,
            },
        });
    }

    let mut ghr  = GlobalHistoryRegister::new(64);
    let mut tage = tage_cfg.build();

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

