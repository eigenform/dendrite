
use bitvec::prelude::*;
use crate::history::*;
use crate::direction::*;
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct TAGEEntry {
    ctr: SaturatingCounter,
    useful: u8,
    tag: Option<usize>,
}
impl TAGEEntry { 
    pub fn new() -> Self { 
        Self { 
            ctr: SaturatingCounter::new(-4..=4, 0, Outcome::N),
            useful: 0,
            tag: None,
        }
    }

    pub fn reset(&mut self) {
        self.ctr.reset();
        self.useful = 0;
        self.tag = None;
    }
}


/// Describing the parameters of a tagged component.
#[derive(Clone, Debug)]
pub struct TAGEComponentParams {
    /// Number of entries in this component.
    size: usize,

    /// Range of global history bits used for tagging entries.
    ghr_range: RangeInclusive<usize>,

    /// The size of a tag [in bits].
    tag_bits: usize,

    /// A function used to select relevant bits from a program counter value.
    pc_sel: fn(usize) -> usize,
}
impl TAGEComponentParams {
    pub fn new(
        size: usize, 
        ghr_range: RangeInclusive<usize>, 
        tag_bits: usize,
        pc_sel: fn(usize) -> usize,
    ) -> Self
    {
        Self { size, ghr_range, tag_bits, pc_sel }
    }

    pub fn build(self) -> TAGEComponent { 
        assert!(self.size != 0);
        assert!(self.ghr_range != (0..=0));
        TAGEComponent { 
            data: vec![TAGEEntry::new(); self.size],
            p: self,
        }
    }
}

/// A tagged component in the TAGE predictor. 
#[derive(Clone, Debug)]
pub struct TAGEComponent {
    data: Vec<TAGEEntry>,
    p: TAGEComponentParams,
}
impl TAGEComponent { 
    /// Get a reference to an entry matching the given tag.
    pub fn lookup(&self, tag: usize) -> Option<&TAGEEntry> {
        self.data.iter().find(|e| {
            if let Some(t) = e.tag { t == tag } else { false }
        })
    }

    /// Get a mutable reference to an entry matching the given tag.
    pub fn lookup_mut(&mut self, tag: usize) -> Option<&mut TAGEEntry> {
        self.data.iter_mut().find(|e| { 
            if let Some(t) = e.tag { t == tag } else { false }
        })
    }

}


pub struct TAGEPredictor { 
    /// Base predictor
    base: Vec<SaturatingCounter>,
    /// Tagged components
    comp: Vec<TAGEComponent>,
}
impl TAGEPredictor {
    pub fn new() -> Self { 
        let mut res = Self { 
            base: vec![SaturatingCounter::new(-1..=1, 0, Outcome::N); 64],
            comp: Vec::new(),
        };

        // FIXME: Let users define this function
        let pc_sel = |x: usize| x & 0x3ff;

        res.add_component(TAGEComponentParams::new(64, 0..=127, 10, pc_sel));
        res.add_component(TAGEComponentParams::new(64, 0..=63,  10, pc_sel));
        res.add_component(TAGEComponentParams::new(64, 0..=31,  10, pc_sel));
        res.add_component(TAGEComponentParams::new(64, 0..=15,  10, pc_sel));

        res
    }

    /// Add a new tagged component to the predictor. 
    fn add_component(&mut self, p: TAGEComponentParams) {
        self.comp.push(p.build());
    }

    /// Given some (a) reference to a [TAGEComponent], (b) reference to some 
    /// [GlobalHistoryRegister], and (c) a program counter value; compute the 
    /// value of a tag which will be used to access the given [TAGEComponent].
    fn create_tag(comp: &TAGEComponent, ghr: &GlobalHistoryRegister, pc: usize)
        -> usize 
    { 
        let ghist_bits = ghr.fold(
            comp.p.ghr_range.clone(), comp.p.tag_bits
        );
        let pc_bits = (comp.p.pc_sel)(pc);
        ghist_bits ^ pc_bits
    }

    // FIXME: Right now, we're relying on the fact that tagged components 
    // are added to `self.comp` in order from longest history to shortest. 
    // In TAGE, matches in components for longer history take precedence. 

    /// Search all tagged components for a matching entry. 
    /// Returns a reference to the entry [if one exists].
    fn lookup(&self, ghr: &GlobalHistoryRegister, pc: usize) 
        -> Option<&TAGEEntry>
    {
        let mut entry: Option<&TAGEEntry> = None;
        for comp in self.comp.iter() {
            let tag = Self::create_tag(&comp, ghr, pc);
            let res = comp.lookup(tag);
            if res.is_some() { 
                entry = res;
            }
        }
        entry
    }

    /// Search all tagged components for a matching entry. 
    /// Returns a mutable reference to the entry [if one exists].
    fn lookup_mut(&mut self, ghr: &GlobalHistoryRegister, pc: usize) 
        -> Option<&mut TAGEEntry>
    {
        let mut entry: Option<&mut TAGEEntry> = None;
        for comp in self.comp.iter_mut() {
            let tag = Self::create_tag(&comp, ghr, pc);
            let res = comp.lookup_mut(tag);
            if res.is_some() { 
                entry = res;
            }
        }
        entry
    }

}

