
use bitvec::prelude::*;
use std::ops::{ RangeInclusive };


pub struct HistoryRegister {
    pub data: BitVec<usize, Lsb0>,
    len: usize,
}

// NOTE: This *reverses* the all of the bits and presents them in a format 
// where the leftmost bit is the most-significant (index n) and the rightmost 
// bit is the least-significant (index 0).
impl std::fmt::Display for HistoryRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let x: String = self.data.as_bitslice().iter().by_vals()
            .map(|b| if b { '1' } else { '0' })
            .rev()
            .collect();
        write!(f, "{}", x)
    }
}

impl HistoryRegister {
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


impl HistoryRegister {
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

/// A circular shift register used to track folded history.
///
/// This folds some global history into 'size' bits, but without the need to
/// actually read all of the history bits and fold them all together with XOR.
/// The result should be equivalent to using [HistoryRegister::fold].
///
/// This strategy is supposed to mirror the hardware implementation described 
/// in "BADGR: A Practical GHR Implementation for TAGE Branch Predictors"
/// (Schlais and Lipasti, 2016).
///
/// NOTE: I think this is only relevant if you're shifting in a single bit. 
/// You'd have to rewrite this if you want to use some other strategy. 
///
#[derive(Clone, Debug)]
pub struct FoldedHistoryRegister {
    data: BitVec,

    /// The size of the output [in bits].
    output_size: usize,

    /// The range of bits in global history to-be-folded.
    ghist_range: RangeInclusive<usize>,
}
impl FoldedHistoryRegister { 
    pub fn new(output_size: usize, ghist_range: RangeInclusive<usize>) 
        -> Self 
    {
        Self { 
            data: bitvec![0; output_size],
            output_size, 
            ghist_range,
        }
    }

    /// Return the folded history as a [BitSlice].
    pub fn output(&self) -> &BitSlice { self.data.as_bitslice() }

    /// Return the folded history as a [usize].
    pub fn output_usize(&self) -> usize { self.data.load::<usize>() }

    /// Using some [HistoryRegister], update the folded history.
    pub fn update(&mut self, ghr: &HistoryRegister) {

        let slice = &ghr.data()[self.ghist_range.clone()];
        let ghist_size = self.ghist_range.end() - self.ghist_range.start();

        let index = ghist_size % self.output_size;

        let newest_bit = *slice.first().unwrap();
        let oldest_bit = *slice.last().unwrap();
        let first_bit  = newest_bit ^ self.data[0];
        let last_bit   = oldest_bit ^ self.data[index];

        // Rotate by one bit
        self.data.rotate_right(1);

        // The newest relevant history bit is XOR'ed with with the first bit 
        self.data.set(0, first_bit);

        // The last relevant history bit will be XOR'ed with this bit
        self.data.set(index, last_bit);
    }
}


