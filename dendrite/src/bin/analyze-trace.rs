
use dendrite::*;
use itertools::*;
use std::env;
use std::collections::*;

fn analyze_branches(trace: &BinaryTrace) {
    let trace_records = trace.as_slice();

    // Extract data from all conditional branches
    let mut stat = TraceStats::new();
    for record in trace_records.iter().filter(|r| r.is_conditional()) {
        let entry = stat.get_mut(record.pc);
        entry.outcomes.push(record.outcome());
    }
    println!("[*] Found {} conditional branches", stat.num_unique_branches());

    let mut unk_brns = Vec::new();

    let mut class_map: HashMap<BranchClass, (usize, usize)> = HashMap::new();
    for (pc, brn) in stat.data.iter().sorted_by(|x, y| {
        x.1.outcomes.len().partial_cmp(&y.1.outcomes.len()).unwrap()
    })
    {
        let class = brn.classify();
        if class == BranchClass::Unknown {
            unk_brns.push((pc, brn));
        }
        let entry = class_map.entry(class).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += brn.outcomes.len();
    }

    println!("[*] Classified branches into {} bins", class_map.len());

    println!(" {:<12} | {:<12} | {:<}", "Branches", "Outcomes", "Classification");
    println!("--------------+--------------+-----------------------");
    for (class, (num_branches, num_outcomes)) in class_map.iter() 
        .sorted_by(|x, y| x.1.partial_cmp(&y.1).unwrap()).rev()
    {
        println!(" {:<12} | {:<12} | {:<?}", num_branches, num_outcomes, class);
    }
    println!();

    println!("[*] There are {} unclassified branches ", unk_brns.len());
    for (pc, brn) in unk_brns {
        let pairs = brn.outcomes.into_pairs();
        println!("{:016x}: {} pairs", pc, pairs.len());
        //brn.dump_info();
        //println!("{:?}", dict);

        //for p in pairs.chunks(8) {
        //    println!("{:?}", p);
        //}
        //println!();
    }


    //let mut behavior_map: HashMap<BranchClass, (usize, usize)> 
    //    = HashMap::new();
    //let mut unks: Vec<(usize, BranchData)> = Vec::new();

    //for (pc, brn) in stat.data.iter().sorted_by(|x, y| {
    //    x.1.outcomes.len().partial_cmp(&y.1.outcomes.len()).unwrap()
    //})
    //{
    //    //if brn.outcomes.len() < 8 { continue; }
    //    let behavior = brn.outcomes.characterize();
    //    let e = behavior_map.entry(behavior.clone()).or_insert((0,0));
    //    e.0 += 1;
    //    e.1 += brn.outcomes.len();
    //}

    //println!("-------------+--------------+-------------------");
    //println!("{:12} | {:12} | {:12}", "Unique", "Total", "Behavior");
    //println!("-------------+--------------+-------------------");
    //for (behavior, (uniqs, brns)) in behavior_map.iter()
    //    .sorted_by(|x, y| x.1.1.partial_cmp(&y.1.1).unwrap()).rev()
    //{
    //    //if *uniqs == 1 { continue; }
    //    println!("{:12} | {:12} | {:?}", uniqs, brns, behavior);
    //}



}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <trace file>", args[0]);
        return;
    }

    let trace = BinaryTrace::from_file(&args[1], "");
    println!("[*] Loaded {} records from '{}'", trace.as_slice().len(), args[1]);
    analyze_branches(&trace);
}

