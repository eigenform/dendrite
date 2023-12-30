
use dendrite::*;
use dendrite::stats::*;
use itertools::*;
use std::env;
use std::time::Instant;
use bitvec::prelude::*;


/// Fold a program counter value into 12 bits. 
///
/// NOTE: I get the impression that this is unreasonably effective, but then
/// again, we're folding 36 bits of the program counter, and this is probably
/// expensive in hardware? 
fn fold_pc_12b(pc: usize) -> usize { 
    let lo  = pc & 0b0000_0000_0000_0000_0000_0000_1111_1111_1111;
    let hi  = pc & 0b0000_0000_0000_1111_1111_1111_0000_0000_0000 >> 12;
    let hi2 = pc & 0b1111_1111_1111_0000_0000_0000_0000_0000_0000 >> 24;
    lo ^ hi ^ hi2
}

/// Index function into the base component.
fn tage_base_fold_pc_12b(comp: &TAGEBaseComponent, pc: usize) -> usize { 
    fold_pc_12b(pc)
}

/// Index function into a tagged component. 
/// - 12 bits from the folded program counter value
/// - 12 bits from the path history register
/// - 12 bits from the folded global history register
fn tage_fold_phr_ghist_12b(comp: &TAGEComponent, 
    pc: usize, phr: &HistoryRegister) -> usize
{
    let pc_bits = fold_pc_12b(pc);
    let phr_bits = phr.data()[0..=11].load::<usize>() & 0b1111_1111_1110;

    let ghist_bits = comp.csr.output_usize(); 
    ghist_bits ^ pc_bits ^ phr_bits

}

/// Hash function for computing a tag.
fn tage_compute_tag(comp: &TAGEComponent, pc: usize) -> usize { 
    let pc_bits = fold_pc_12b(pc);
    let ghist0_bits = comp.csr.output_usize();
    let ghist1_bits = comp.csr.output_usize() << 1;
    (pc_bits ^ ghist0_bits ^ ghist1_bits) & ((1 << comp.cfg.tag_bits) -1)
}

/// Update the path history register. 
/// - Fold program counter into 12 bits
/// - Shift the PHR by one
/// - XOR the folded program counter with the low 12 bits in the PHR
fn update_phr(pc: usize, phr: &mut HistoryRegister) {
    let pc_bits  = fold_pc_12b(pc);

    phr.shift_by(1);
    let phr_bits = phr.data()[0..=11].load::<usize>();
    let new_bits = pc_bits ^ phr_bits;
    phr.data_mut()[0..=11].store(new_bits);
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
            index_strat: IndexStrategy::FromPc(tage_base_fold_pc_12b),
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
                    max_t_state: 1,
                    max_n_state: 1,
                    default_state: Outcome::N,
                },
                index_strat: IndexStrategy::FromPhr(tage_fold_phr_ghist_12b),
                tag_strat: TagStrategy::FromPc(tage_compute_tag),
        });
    }

    //println!("[*] {:#?}", tage_cfg);
    println!("[*] TAGE entries (in total): {}", tage_cfg.total_entries());

    let storage_bits  = tage_cfg.storage_bits();
    let storage_kib = storage_bits as f64 / 1024.0 / 8.0;
    println!("[*] TAGE storage bits: {}b, {:.2}KiB", 
        storage_bits, storage_kib
    );

    tage_cfg.build()
}

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1]);
    let trace_records = trace.as_slice();
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

    let ghr_bits = 128;
    let mut tage = build_tage();
    let mut ghr  = HistoryRegister::new(ghr_bits);
    let mut phr  = HistoryRegister::new(32);
    println!("[*] GHR length: {}", ghr_bits);

    // Randomize the state of global history before we start evaluating 
    for _ in 0..64 {
        ghr.shift_by(1);
        ghr.data_mut().set(0, rand::random());
        tage.update_history(&ghr);

        phr.shift_by(1);
        phr.data_mut().set(0, rand::random());

    }

    let mut hits = 0;
    let mut brns = 0;
    let mut mpkb_cnts = Vec::new();
    let mut mpkb_window = 0;
    let mut stats = BranchStats::new();

    let start = Instant::now();
    for record in trace_records {
        match record.kind { 
            BranchKind::Invalid => unreachable!(),

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
                update_phr(record.pc, &mut phr);
            },

            // Use the TAGE predictor to evaluate conditional branches
            BranchKind::DirectBranch => {
                if brns % 1000 == 0 { 
                    mpkb_cnts.push(mpkb_window);
                    mpkb_window = 0;
                }

                let stat = stats.get_mut(record.pc);
                stat.pat.push(record.outcome.into());

                let inputs = TAGEInputs { 
                    pc: record.pc,
                    phr: &phr,
                };
                let p = tage.predict(inputs.clone());
                if record.outcome == p.outcome {
                    hits += 1;
                    stat.hits += 1;
                } else { 
                    mpkb_window += 1;
                }
                stat.occ += 1;
                brns += 1;

                tage.update(inputs, p, record.outcome);
                ghr.shift_by(1);
                ghr.data_mut().set(0, record.outcome.into());
                tage.update_history(&ghr);
                update_phr(record.pc, &mut phr);
            },
        }
    }
    let done = start.elapsed();
    println!("[*] Completed in {:.3?}", done);
    println!("[*] {:#?}", tage.stat);

    println!("[*] Unique branches: {}", stats.num_unique_branches());
    let hit_rate = hits as f64 / brns as f64; 
    println!("[*] Global hit rate: {}/{} ({:.2}% correct) ({} misses)", 
        hits, brns, hit_rate*100.0, brns - hits);
    let avg_mpkb = mpkb_cnts.iter().sum::<usize>() / mpkb_cnts.len();
    println!("[*] Average MPKB:    {}/1000 ({:.4})", 
        avg_mpkb, avg_mpkb as f64 / 1000.0);

    for (idx, comp) in tage.comp.iter().enumerate() {
        println!("[*] Component[{}] (GHR[{:03?}]): {:.2}% utilization", 
            idx, comp.cfg.ghr_range, comp.utilization());
    }


    let d: Vec<(&usize, &BranchData)> = stats.data.iter().collect();

    println!("[*] Low hit rate branches:");
    let low_rate_iter = d.iter()
        .filter(|(_, s)| s.occ > 100 && s.hit_rate() <= 0.55)
        //.sorted_by(|x,y| { x.1.hit_rate().partial_cmp(&y.1.hit_rate()).unwrap() })
        .sorted_by(|x,y| { x.1.occ.partial_cmp(&y.1.occ).unwrap() }).rev();
    for (pc, s) in low_rate_iter {
        let pat = if s.pat.len() > 64 {
            let slice = &s.pat.as_bitslice()[0..64];
            format!("{:b}", slice)
        } else {
            format!("{:b}", s.pat)
        };
        println!("    {:016x}: {:6}/{:6} ({:.4}) {}",
            pc, s.hits, s.occ, s.hit_rate(), pat);
    }

}


