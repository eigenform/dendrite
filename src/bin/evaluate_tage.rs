
use dendrite::*;
use std::env;
use itertools::*;

// Fold the program counter into a 12-bit index
fn tage_get_base_pc(pc: usize) -> usize { 
    let lo  = pc & 0b0000_0000_0000_0000_0000_0000_1111_1111_1111;
    let hi  = pc & 0b0000_0000_0000_1111_1111_1111_0000_0000_0000 >> 12;
    let hi2 = pc & 0b1111_1111_1111_0000_0000_0000_0000_0000_0000 >> 24;
    lo ^ hi ^ hi2
}

// Fold the program counter into a 16-bit index
fn btb_get_base_pc(pc: usize) -> usize {
    let lo  = pc & 0b0000_0000_0000_0000_1111_1111_1111_1111;
    let hi  = pc & 0b1111_1111_1111_1111_0000_0000_0000_0000 >> 16;
    lo ^ hi
}

fn build_tage() -> TAGEPredictor {
    let mut tage_cfg = TAGEConfig::new(
        TAGEBaseConfig { 
            ctr: SaturatingCounterConfig {
                max_t_state: 2,
                max_n_state: 2,
                default_state: Outcome::N,
            },
            size: 1 << 12,
            index_fn: tage_get_base_pc,
        },
    );

    for ghr_range_hi in &[7, 15, 31, 63, 127] {
        tage_cfg.add_component(TAGEComponentConfig {
            size: 1 << 12,
            ghr_range: 0..=*ghr_range_hi,
            tag_bits: 12,
            pc_sel_fn: tage_get_base_pc,
            ctr: SaturatingCounterConfig {
                max_t_state: 2,
                max_n_state: 2,
                default_state: Outcome::N,
            },
        });
    }

    tage_cfg.build()
}

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1]);
    let trace_records = trace.as_slice_trunc(2_000_000);
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    let mut btb = SimpleBTB::new(1 << 16, btb_get_base_pc);
    let mut ghr  = GlobalHistoryRegister::new(128);
    let mut tage = build_tage();

    // Randomize the state of global history before we start evaluating 
    for _ in 0..64 {
        ghr.shift_by(1);
        ghr.data_mut().set(0, rand::random());
        tage.update_history(&ghr);
    }

    let mut hits = 0;
    let mut brns = 0;
    let mut mpkb_cnts = Vec::new();
    let mut mpkb_window = 0;
    let mut stats = BranchStats::new();

    for record in trace_records {

        match record.kind { 
            BranchKind::DirectBranch => {},
            BranchKind::DirectJump |
            BranchKind::IndirectJump |
            BranchKind::DirectCall |
            BranchKind::IndirectCall |
            BranchKind::Return => {
                ghr.shift_by(1);
                ghr.data_mut().set(0, true);
                tage.update_history(&ghr);
            },
            BranchKind::Invalid => unimplemented!("???"),
        }

        if let BranchKind::DirectBranch = record.kind { 
            let stat = stats.get_mut(record.pc);
            stat.occ += 1;

            //println!("{:016x?}", record);
            if brns % 1000 == 0 { 
                mpkb_cnts.push(mpkb_window);
                mpkb_window = 0;
            }

            brns += 1;
            let p = tage.predict(record.pc);
            if record.outcome == p.outcome {
                hits += 1;
                stat.hits += 1;
            } else { 
                mpkb_window += 1;
            }
            tage.update(record.pc, p, record.outcome);
            ghr.shift_by(1);
            ghr.data_mut().set(0, record.outcome.into());
            tage.update_history(&ghr);
        }
    }

    let hit_rate = hits as f64 / brns as f64; 
    let avg_mpkb = mpkb_cnts.iter().sum::<usize>() / mpkb_cnts.len();
    println!("[*] Unique branches: {}", stats.num_branches());
    println!("[*] Global hit rate: {}/{} ({:.2}%)", 
        hits, brns, hit_rate*100.0);
    println!("[*] Average MPKB:    {}/1000 ({:.4})", 
        avg_mpkb, avg_mpkb as f64 / 1000.0);

    let d: Vec<(&usize, &BranchData)> = stats.data.iter().collect();

    println!("[*] Low hit rate branches:");
    for (pc, s) in d.iter()
        .filter(|(pc, s)| s.occ > 100 && s.hit_rate() <= 0.55)
        .sorted_by(|x,y| { 
            x.1.hit_rate().partial_cmp(&y.1.hit_rate()).unwrap()
        })
    {
        println!("{:016x}: {:6}/{:6} ({:.4})", 
            pc, s.hits, s.occ, s.hit_rate());
    }



}


