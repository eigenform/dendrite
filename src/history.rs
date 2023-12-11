
use bitvec::prelude::*;
use std::ops::{ RangeInclusive };


pub struct GlobalHistoryRegister {
    pub data: BitVec<usize, Lsb0>,
    len: usize,
}

// NOTE: This *reverses* the all of the bits and presents them in a format 
// where the rightmost bit is the most-significant (index n) and the leftmost 
// bit is the least-significant (index 0).
impl std::fmt::Display for GlobalHistoryRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let x: String = self.data.as_bitslice().iter().by_vals()
            .map(|b| if b { '1' } else { '0' })
            .rev()
            .collect();
        write!(f, "{}", x)
    }
}

impl GlobalHistoryRegister {
    /// Create a register with the specified length in bits.
    /// All bits in the register are initialized to zero. 
    pub fn new(len: usize) -> Self { 
        let mut res = Self { 
            data: bitvec![usize, Lsb0; 0; len],
            len,
        };
        res
    }

    pub fn len(&self) -> usize { self.len }
    pub fn data(&self) -> &BitVec { &self.data }
    pub fn data_mut(&mut self) -> &mut BitVec { &mut self.data }
}


impl GlobalHistoryRegister {
    /// Shift the register by 'n' bits. 
    /// The bottom 'n' bits become zero, and the top 'n' bits are discarded.
    pub fn shift_by(&mut self, n: usize)  {
        self.data.shift_right(n);
    }

    /// Return some slice of bits.
    pub fn read(&self, range: RangeInclusive<usize>) -> &BitSlice {
        &self.data[range]
    }

    /// Fold [with XOR] some slice of bits. 
    pub fn fold(&self, range: RangeInclusive<usize>, output_bits: usize)
        -> usize 
    { 
        let output_mask = (1 << output_bits) - 1;
        let slice = &self.data[range];
        let chunks = slice.chunks(output_bits);
        let res = chunks.fold(0, |mut res, x| {
            let val = x.load::<usize>();
            //println!("Fold {:012b} ^ {:012b}", res, val);
            res ^= val;
            res
        });

        res & output_mask
    }

}





