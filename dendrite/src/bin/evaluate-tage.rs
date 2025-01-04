
use dendrite::*;
use dendrite::stats::*;
use itertools::*;
use std::env;
use std::time::Instant;
use bitvec::prelude::*;

fn build_tage() -> TAGEPredictor {
    let mut tage_cfg = TAGEConfig::new(
        TAGEBaseConfig { 
            ctr: SaturatingCounterConfig {
                max_t_state: 4,
                max_n_state: 4,
                default_state: Outcome::N,
            },
            size: 1 << 12,
            index_strat: IndexStrategy::FromPc(|component, pc| {
                pc & 0b1111_1111_1111
            }),
        },
    );

    for ghr_range_hi in &[7, 15, 31, 63, 127] {
        tage_cfg.add_component(
            TAGEComponentConfig {
                size: 1 << 12,
                ghr_range: 0..=*ghr_range_hi,
                tag_bits: 8,
                useful_bits: 1,
                ctr: SaturatingCounterConfig {
                    max_t_state: 4,
                    max_n_state: 4,
                    default_state: Outcome::N,
                },
                index_strat: IndexStrategy::FromPc(|component, pc| {
                    pc & 0b1111_1111_1111
                }),
                tag_strat: TagStrategy::FromPc(|component, pc| {
                    let pc_bits      = pc & 0b1111_1111;
                    let ghist0_bits  = component.csr.output_usize();
                    let ghist1_bits  = component.csr.output_usize() << 1;
                    let eff_tag_mask = ((1 << component.cfg.tag_bits) - 1);
                    (pc_bits ^ ghist0_bits ^ ghist1_bits) & eff_tag_mask
                }),
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

    let trace = BinaryTrace::from_file(&args[1], "");
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    let ghr_bits = 128;
    let mut tage = build_tage();
    let mut ghr  = HistoryRegister::new(ghr_bits);
    println!("[*] TAGE configuration:");
    println!("      Entries (in total): {}", tage.cfg.total_entries());
    println!("        {} entries (base component)", tage.cfg.base.size);
    for idx in 0..tage.cfg.comp.len() {
        println!("        {} entries (tagged component {})", 
            tage.cfg.comp[idx].size, idx, 
        );
    }
    let storage_bits  = tage.cfg.storage_bits();
    let storage_kib = storage_bits as f64 / 1024.0 / 8.0;
    println!("      Storage bits: {}b, {:.2}KiB", 
        storage_bits, storage_kib
    );
    println!("      Global History Register (GHR) length: {} bits", ghr_bits);

    // Randomize the state of global history 
    for _ in 0..64 {
        ghr.shift_by(1);
        ghr.data_mut().set(0, rand::random());
        tage.update_history(&ghr);
    }

    let mut hits = 0;
    let mut brns = 0;
    let mut mpkb_cnts = Vec::new();
    let mut mpkb_window = 0;
    let mut stats = TraceStats::new();

    let start = Instant::now();
    for record in trace_records {
        match record.kind() { 
            // Record all unconditionally taken branches in the GHR and 
            // propagate updates to the TAGE folded history registers
            BranchKind::DirectJump |
            BranchKind::IndirectJump |
            BranchKind::DirectCall |
            BranchKind::IndirectCall |
            BranchKind::Return => {
                ghr.shift_by(1);
                ghr.data_mut().set(0, true);
                tage.update_history(&ghr);
            },

            // Use the TAGE predictor to evaluate conditional branches
            BranchKind::DirectBranch => {
                // Sample the number of mispredictions every 1000 branches
                if brns % 1000 == 0 { 
                    mpkb_cnts.push(mpkb_window);
                    mpkb_window = 0;
                }

                let stat = stats.get_mut(record.pc);
                stat.outcomes.push(record.outcome());

                let inputs = TAGEInputs { 
                    pc: record.pc,
                };
                let prediction = tage.predict(inputs.clone());
                if record.outcome() == prediction.outcome {
                    hits += 1;
                    stat.hits += 1;
                } else { 
                    mpkb_window += 1;
                }
                brns += 1;

                tage.update(inputs, prediction, record.outcome());

                // Update the global history register. 
                // Use the GHR to update the folded history registers in 
                // each of the tagged components. 
                ghr.shift_by(1);
                ghr.data_mut().set(0, record.outcome().into());
                tage.update_history(&ghr);
            },
        }
    }
    let done = start.elapsed();
    println!("[*] ... simulated in {:.3?}", done);
    println!();

    let hit_rate = hits as f64 / brns as f64; 
    let avg_mpkb = mpkb_cnts.iter().sum::<usize>() / mpkb_cnts.len();
    println!("[*] Global statistics:");
    println!("      Unique branches: {}", stats.num_unique_branches());
    println!("      Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
        hits, brns, hit_rate*100.0, brns - hits
    );
    println!("      Average MPKB:    {} miss/kbrn", avg_mpkb);
    println!();

    println!("[*] Per-component statistics:");
    println!("      Base component:");
    println!("        {} misses, {} hits", 
        tage.stat.base_miss, tage.stat.base_hits
    );
    for (idx, comp) in tage.comp.iter().enumerate() {
        println!("      Component[{:1}] (GHR[{:03?}]):",
            idx, comp.cfg.ghr_range, 
        );
        println!("        {} misses, {} hits", 
            tage.stat.comp_miss[idx], tage.stat.comp_hits[idx]
        );
        println!("        {:.2}% utilization", comp.utilization());
    }

    println!("Low hit-rate branches:");
    for (pc, data) in stats.get_low_rate_branches(8) {
        println!("  {:016x} {:8}/{:8} {:.4}", 
            pc, data.hits, data.len(), data.hit_rate()
        );
    }

    // Print some information about branches with a low hit rate
    //println!("[*] Low hit rate branches:");
    //let d: Vec<(&usize, &BranchData)> = stats.data.iter().collect();
    //let low_rate_iter = d.iter()
    //    .filter(|(_, s)| s.occ > 100 && s.hit_rate() <= 0.55)
    //    //.sorted_by(|x,y| { x.1.hit_rate().partial_cmp(&y.1.hit_rate()).unwrap() })
    //    .sorted_by(|x,y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() }).rev();
    //for (pc, s) in low_rate_iter {
    //    let pat = if s.outcomes.len() > 64 {
    //        let slice = &s.outcomes.as_bitslice()[0..32];
    //        format!("{:b}", slice)
    //    } else {
    //        format!("{:b}", s.outcomes.as_bitslice())
    //    };
    //    println!("    {:016x}: {:6}/{:6} ({:.4}) {}",
    //        pc, s.hits, s.occ, s.hit_rate(), pat);
    //}

}


