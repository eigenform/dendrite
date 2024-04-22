
use crate::predictor::*;
use std::ops::RangeInclusive;

/// Configuration for a [`TAGEBaseComponent`].
#[derive(Clone, Debug)]
pub struct TAGEBaseConfig {
    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,

    /// Number of entries
    pub size: usize,

    /// Strategy for indexing into the table.
    pub index_strat: IndexStrategy<TAGEBaseComponent>,
}
impl TAGEBaseConfig {
    /// Get the [approximate] number of storage bits. 
    pub fn storage_bits(&self) -> usize { 
        self.ctr.storage_bits() * self.size
    }

    /// Use this configuration to create a new [`TAGEBaseComponent`].
    pub fn build(self) -> TAGEBaseComponent {
        assert!(self.size.is_power_of_two());
        TAGEBaseComponent {
            data: vec![self.ctr.build(); self.size],
            cfg: self,
        }
    }
}

/// Configuration for a [`TAGEComponent`].
#[derive(Clone, Debug)]
pub struct TAGEComponentConfig {
    /// Number of entries
    pub size: usize,

    /// Relevant slice in global history
    pub ghr_range: RangeInclusive<usize>,

    /// Number of tag bits
    pub tag_bits: usize,

    /// Number of bits in the 'useful' counter
    pub useful_bits: usize,

    /// Strategy for indexing into the table
    pub index_strat: IndexStrategy<TAGEComponent>,

    /// Strategy for creating tags
    pub tag_strat: TagStrategy<TAGEComponent>,

    /// Parameters for the saturating counters
    pub ctr: SaturatingCounterConfig,
}
impl TAGEComponentConfig {
    /// Get the [approximate] number of storage bits. 
    pub fn storage_bits(&self) -> usize { 
        let entry_size = (
            self.ctr.storage_bits() +
            self.useful_bits +
            self.tag_bits
        );
        entry_size * self.size
    }

    /// Use this configuration to create a new [`TAGEComponent`].
    pub fn build(self) -> TAGEComponent {
        assert!(self.size.is_power_of_two());
        let csr = FoldedHistoryRegister::new(
            self.size.ilog2() as usize,
            self.ghr_range.clone()
        );
        let entry = TAGEEntry::new(self.ctr.build(), self.useful_bits);
        let data = vec![entry; self.size];

        TAGEComponent {
            cfg: self,
            data,
            csr,
        }
    }
}


/// Configuration for a [`TAGEPredictor`].
#[derive(Clone, Debug)]
pub struct TAGEConfig {
    /// Base component configuration
    pub base: TAGEBaseConfig,

    /// Tagged component configurations
    pub comp: Vec<TAGEComponentConfig>,
}
impl TAGEConfig {
    pub fn new(base: TAGEBaseConfig) -> Self {
        Self {
            base,
            comp: Vec::new(),
        }
    }

    pub fn total_entries(&self) -> usize {
        let c: usize = self.comp.iter().map(|c| c.size).sum();
        self.base.size + c
    }

    /// Get the [approximate] number of storage bits. 
    pub fn storage_bits(&self) -> usize { 
        let c: usize = self.comp.iter().map(|c| c.storage_bits()).sum();
        c + self.base.storage_bits()
    }

    /// Add a tagged component to the predictor.
    pub fn add_component(&mut self, c: TAGEComponentConfig) {
        self.comp.push(c);
        self.comp.sort_by(|x, y| {
            let x_history_len = x.ghr_range.end() - x.ghr_range.start();
            let y_history_len = y.ghr_range.end() - y.ghr_range.start();
            std::cmp::Ord::cmp(&y_history_len, &x_history_len)
        });
    }

    /// Use this configuration to create a new [`TAGEPredictor`].
    pub fn build(self) -> TAGEPredictor {
        let cfg = self.clone();
        let comp = self.comp.iter().map(|c| c.clone().build())
            .collect::<Vec<TAGEComponent>>();
        let base = self.base.build();
        let stat = TAGEStats::new(comp.len());
        TAGEPredictor { 
            cfg, 
            base, 
            comp, 
            stat, 
            reset_ctr: 0,
        }
    }
}



