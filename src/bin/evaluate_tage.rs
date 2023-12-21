
use dendrite::*;
use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1]);
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    // Index function used to select relevant PC bits. 
    let get_base_pc = |x: usize| { 
        let lo = x & 0b0000_0000_0000_0000_0000_1111_1111_1111;
        lo
    };

    let mut tage_cfg = TAGEConfig::new(
        TAGEBaseConfig { 
            ctr: SaturatingCounterConfig {
                max_t_state: 4,
                max_n_state: 4,
                default_state: Outcome::N,
            },
            size: 4096,
            index_fn: get_base_pc,
        },
    );

    for ghr_range_hi in &[7, 15, 31, 63] {
        tage_cfg.add_component(TAGEComponentConfig {
            size: 4096,
            ghr_range: 0..=*ghr_range_hi,
            tag_bits: 12,
            pc_sel_fn: get_base_pc,
            ctr: SaturatingCounterConfig {
                max_t_state: 4,
                max_n_state: 4,
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

    let trace_records = trace.as_slice();
    let mut hits = 0;
    let mut brns = 0;

    let mut mpki_cnts = Vec::new();
    let mut mpki_window = 0;

    for record in trace_records {

        // Only conditional branches for now
        if let BranchKind::DirectBranch = record.kind { 
            //println!("{:016x?}", record);
            if brns % 1000 == 0 { 
                mpki_cnts.push(mpki_window);
                mpki_window = 0;
            }

            brns += 1;
            let p = tage.predict(record.pc);
            if record.outcome == p.outcome {
                hits += 1;
            } else { 
                mpki_window += 1;
            }
            tage.update(record.pc, p, record.outcome);
            ghr.shift_by(1);
            ghr.data_mut().set(0, record.outcome.into());
            tage.update_history(&ghr);
        }


    }

    let hit_rate = hits as f64 / brns as f64; 
    let avg_mpki = mpki_cnts.iter().sum::<usize>() / mpki_cnts.len();
    println!("[*] Global hit rate: {}/{} ({:.2}%)", hits, brns, hit_rate*100.0);
    println!("[*] Average MPKI {}/1000 ({:.4})", avg_mpki, 
        avg_mpki as f64 / 1000.0);



}


