
use dendrite::*;
use bitvec::prelude::*;
use dendrite::tage::*;


fn main() {
    let mut ghr  = GlobalHistoryRegister::new(128);

    let get_base_pc = |x: usize| { x & 0xfff };
    let mut base_component = TAGEBaseComponent::new(
        SaturatingCounter::new(-4..=4, 0, Outcome::N),
        64, get_base_pc
    );

    let foo = |x: usize| { x & 0xfff };
    let mut tage = TAGEPredictor::new(base_component);
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(-4..=4, 0, Outcome::N)), 64, 
        0..=15, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(-4..=4, 0, Outcome::N)), 64, 
        0..=31, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(-4..=4, 0, Outcome::N)), 64, 
        0..=63, 12, foo
    ));
    tage.add_component(TAGEComponent::new(
        TAGEEntry::new(SaturatingCounter::new(-4..=4, 0, Outcome::N)), 64, 
        0..=7, 12, foo
    ));


    //for c in tage.comp.iter() {
    //    println!("{:?}", c);
    //}



    ghr.data_mut()[0..=15].store(0xdead);
    ghr.shift_by(16);
    ghr.data_mut()[0..=15].store(0x1515);
    ghr.shift_by(16);
    ghr.data_mut()[0..=15].store(0xaaaa);
    ghr.shift_by(16);
    ghr.data_mut()[0..=15].store(0x8401);
    ghr.shift_by(16);
    ghr.data_mut()[0..=15].store(0x1111);
    ghr.shift_by(3);

    println!("{}", ghr);
    println!("{:012b}", ghr.fold(0..=64, 12));

    let pc = 0x1000_0000;
    let p = tage.predict(pc);
    println!("{:?}", p);

}


