
use dendrite::*;
use bitvec::prelude::*;
use dendrite::tage::*;


fn main() {
    let mut ghr  = GlobalHistoryRegister::new(128);
    let mut tage = TAGEPredictor::new();

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

}




