
use dendrite::*;
use std::env;
use std::collections::*;
use itertools::*;

struct BranchStats { 
    pub data: BTreeMap<usize, BranchData>,
}
impl BranchStats {
    pub fn new() -> Self {
        Self { 
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, pc: usize) -> Option<&BranchData> {
        self.data.get(&pc)
    }
    pub fn get_mut(&mut self, pc: usize) -> &mut BranchData {
        self.data.entry(pc).or_insert(BranchData { occ: 0, hits: 0 })
    }
    pub fn num_branches(&self) -> usize { 
        self.data.len()
    }
}

struct BranchData { 
    pub occ: usize,
    pub hits: usize,
}
impl BranchData {
    pub fn hit_rate(&self) -> f64 {
        self.hits as f64 / self.occ as f64
    }
}

fn build_tage() -> TAGEPredictor {

    // Index function used to select relevant PC bits. 
    // The number of bits should correspond to the size of the tables.
    let get_base_pc = |x: usize| { 
        let lo  = x & 0b0000_0000_0000_0000_0000_0000_1111_1111_1111;
        let hi  = x & 0b0000_0000_0000_1111_1111_1111_0000_0000_0000 >> 12;
        let hi2 = x & 0b1111_1111_1111_0000_0000_0000_0000_0000_0000 >> 24;
        lo ^ hi ^ hi2
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

    for ghr_range_hi in &[7, 15, 31, 63, 127] {
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
    println!("[*] Loaded {} records from {}", trace.num_entries(), args[1]);

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


