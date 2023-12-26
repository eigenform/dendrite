
pub mod component;
pub use component::*;

use bitvec::prelude::*;
use rand::distributions::{ WeightedIndex, Distribution };

use crate::history::*;
use crate::Outcome;
use crate::predictor::*;

/// Container for inputs passed to TAGE components.
#[derive(Clone)]
pub struct TAGEInputs<'a> { 
    /// Program counter associated with a predicted branch
    pub pc: usize,

    /// Bits from a path history register
    pub phr: &'a HistoryRegister,
}


/// Identifies a particular TAGE component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TAGEProvider { 
    /// The base component
    Base, 

    /// A tagged component
    Tagged(usize), 
}

/// The output from [TAGEPredictor::predict] including the predicted outcome
/// and other metadata about how the prediction was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TAGEPrediction {
    /// The component providing the prediction
    pub provider: TAGEProvider,

    /// A predicted direction
    pub outcome: Outcome,

    /// The index identifying the entry used to make this prediction
    pub idx: usize,

    /// The tag matching the entry used to make this prediction
    pub tag: usize,

    /// Alternate component used to provide a prediction
    pub alt_provider: TAGEProvider,

    /// Predicted direction from the alternate component
    pub alt_outcome: Outcome,

    /// The index identifying the entry from the alternate component
    pub alt_idx: usize,

    /// The tag matching the entry from the alternate component
    pub alt_tag: usize,
}

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

    pub fn storage_bits(&self) -> usize { 
        let c: usize = self.comp.iter().map(|c| c.storage_bits()).sum();
        c + self.base.storage_bits()
    }

    pub fn add_component(&mut self, c: TAGEComponentConfig) {
        self.comp.push(c);
        self.comp.sort_by(|x, y| {
            let x_history_len = x.ghr_range.end() - x.ghr_range.start();
            let y_history_len = y.ghr_range.end() - y.ghr_range.start();
            std::cmp::Ord::cmp(&y_history_len, &x_history_len)
        });
    }

    pub fn build(self) -> TAGEPredictor {
        let cfg = self.clone();
        let comp = self.comp.iter().map(|c| c.clone().build())
            .collect::<Vec<TAGEComponent>>();
        let base = self.base.build();
        let stat = TAGEStats::new(comp.len());
        TAGEPredictor { cfg, base, comp, stat, 
            reset_ctr: 0,
        }
    }
}

#[derive(Debug)]
pub struct TAGEStats {
    pub alcs: usize,
    pub failed_alcs: usize,
    pub base_miss: usize,
    pub comp_miss: Vec<usize>,
    pub resets: usize,
}
impl TAGEStats {
    pub fn new(num_comp: usize) -> Self { 
        Self {
            alcs: 0,
            failed_alcs: 0,
            base_miss: 0,
            comp_miss: vec![0; num_comp],
            resets: 0,
        }
    }
}



/// The "TAgged GEometric history length" predictor. 
///
/// See the following: 
///  - "A case for (partially) TAgged GEometric history length branch prediction" 
///  (Seznec, 2006).
pub struct TAGEPredictor { 
    pub cfg: TAGEConfig,

    pub stat: TAGEStats,

    /// Base component
    pub base: TAGEBaseComponent,

    /// Tagged components
    pub comp: Vec<TAGEComponent>,

    pub reset_ctr: u8,
}
impl TAGEPredictor {

    /// Given a program counter value and the provider of an incorrect 
    /// prediction, try to select a tagged component that will be used to 
    /// allocate a new entry. 
    fn alloc(&self, input: TAGEInputs, provider: TAGEProvider) 
        -> Option<usize>
    { 
        // Early return: when the provider is the component with the longest 
        // associated history length, we cannot allocate.
        //
        // NOTE: Remember that the provider with the longest history can 
        // always be found at index 0.
        if matches!(provider, TAGEProvider::Tagged(0)) {
            return None;
        }

        // Get the indexes of all components whose associated history length
        // is longer than the provider. 
        let provider_range = match provider { 
            TAGEProvider::Base => 0..=self.shortest_tagged_component(),
            TAGEProvider::Tagged(idx) => 0..=(idx-1),
        };

        // A component is only eligible when the entry associated with this
        // program counter has its 'useful' bits set to zero. 
        let mut candidates: Vec<usize> = Vec::new();
        for idx in provider_range {
            let index = self.comp[idx].get_index(input.clone());
            let entry = self.comp[idx].get_entry(index);
            if entry.useful == 0 {
                candidates.push(idx);
            }
        }

        // Easy case: we failed allocate a new entry
        if candidates.is_empty() {
            return None;
        }

        // Easy case: there's only a single candidate.
        if candidates.len() == 1 {
            return candidates.first().copied();
        } 

        // Otherwise, we need some strategy for selecting between multiple 
        // candidates. In the original paper, the probability scales *down* 
        // with candidates of increasing history length: given candidates with 
        // history lengths J and K (where J < K), the candidate J is twice as 
        // likely to be chosen over K.
        let mut rng = rand::thread_rng();
        let weights: Vec<usize> = candidates.iter().map(|idx| 1 << idx)
            .collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        Some(candidates[dist.sample(&mut rng)])
    }

    fn update_incorrect(&mut self, 
        input: TAGEInputs, 
        prediction: TAGEPrediction, 
        outcome: Outcome
    )
    {
        // Update the entry in the component that provided the prediction
        match prediction.provider {
            TAGEProvider::Base => {
                self.stat.base_miss += 1;
                let index = self.base.get_index(input.clone());
                let entry = self.base.get_entry_mut(index);
                entry.update(outcome);
            },
            TAGEProvider::Tagged(idx) => {
                self.stat.comp_miss[idx] += 1;
                let index = self.comp[idx].get_index(input.clone());
                let entry = self.comp[idx].get_entry_mut(index);
                entry.ctr.update(outcome);
                //entry.decrement_useful();
            },
        }

        // Try to allocate a new entry 
        if let Some(idx) = self.alloc(input.clone(), prediction.provider) {
            let new_index = self.comp[idx].get_index(input.clone());
            let new_tag   = self.comp[idx].get_tag(input.clone());
            let new_entry = self.comp[idx].get_entry_mut(new_index);
            new_entry.invalidate();
            new_entry.tag = Some(new_tag);
            new_entry.useful = 0;
            new_entry.ctr.set_direction(outcome);
            self.stat.alcs += 1;
            self.reset_ctr = self.reset_ctr.saturating_add(1);
        } 
        // Otherwise, use some strategy to age all of the entries
        else { 
            self.stat.failed_alcs += 1;
            self.reset_ctr = self.reset_ctr.saturating_sub(1);
        }

    }

    fn update_correct(&mut self,
        input: TAGEInputs,
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        // Update the entry in the component that provided the prediction
        match prediction.provider {
            TAGEProvider::Base => {
                let index = self.base.get_index(input.clone());
                let entry = self.base.get_entry_mut(index);
                entry.update(outcome);
            },

            // Increment when the alternate prediction is incorrect
            TAGEProvider::Tagged(idx) => {
                let index = self.comp[idx].get_index(input.clone());
                let entry = self.comp[idx].get_entry_mut(index);

                if prediction.alt_outcome != outcome {
                    entry.increment_useful();
                }

                entry.ctr.update(outcome);
            },
        }
    }

    /// Access the base component using the provided input. 
    fn get_base_entry(&self, input: TAGEInputs) 
        -> (usize, &SaturatingCounter)
    {
        let idx = self.base.get_index(input);
        let entry = self.base.get_entry(idx);
        (idx, entry)
    }

    /// Access all tagged components using the provided input. 
    fn get_tagged_entries(&self, input: TAGEInputs) 
        -> Vec<(usize, &TAGEEntry, usize)>
    {
        let mut entries = Vec::new();
        for component in self.comp.iter() {
            let index = component.get_index(input.clone());
            let tag = component.get_tag(input.clone());
            entries.push((index, component.get_entry(index), tag));
        }
        entries
    }

}

/// The public interface to a [TAGEPredictor].
impl TAGEPredictor {
    /// Return the number of tagged components.
    pub fn num_tagged_components(&self) -> usize { 
        self.comp.len()
    }

    /// Return the index of the tagged component with the shortest associated
    /// history length.
    pub fn shortest_tagged_component(&self) -> usize { 
        self.num_tagged_components() - 1
    }

    /// Make a prediction for the provided input. 
    pub fn predict(&self, input: TAGEInputs) -> TAGEPrediction {

        // Access all of the tables
        let (base_idx, base_entry) = self.get_base_entry(input.clone());
        let tagged_entries = self.get_tagged_entries(input.clone());

        let default_outcome = base_entry.predict();

        let mut result = TAGEPrediction {
            provider: TAGEProvider::Base,
            outcome: default_outcome,
            idx: base_idx,
            tag: 0,
            alt_provider: TAGEProvider::Base,
            alt_outcome: default_outcome,
            alt_idx: base_idx,
            alt_tag: 0
        };

        // NOTE: You're iterating through components *backwards* here 
        // (from the shortest to longest history length).
        let tagged_iter = tagged_entries.iter().enumerate().rev();
        for (comp_idx, (entry_idx, entry, tag)) in tagged_iter { 
            if entry.tag_matches(*tag) {
                result.alt_provider = result.provider;
                result.alt_outcome  = result.outcome;
                result.alt_idx = result.idx;
                result.alt_tag = result.tag;

                result.provider = TAGEProvider::Tagged(comp_idx);
                result.outcome  = entry.predict();
                result.idx = *entry_idx;
                result.tag = *tag; 
            }
        }
        result
    }

    /// Given a particular prediction and the resolved outcome, update the 
    /// state of the predictor. 
    ///
    /// NOTE: The paper suggests periodically resetting the 'useful' bits
    /// on all counters (ie. every 256,000 branches?)
    pub fn update(&mut self, 
        input: TAGEInputs, 
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        let misprediction = prediction.outcome != outcome;
        if misprediction {
            self.update_incorrect(input.clone(), prediction, outcome);
        } else {
            self.update_correct(input.clone(), prediction, outcome);
        }

        if self.reset_ctr == u8::MAX {
            self.reset_ctr = 0;
            self.stat.resets += 1;
            for comp in self.comp.iter_mut() {
                comp.reset_useful_bits();
            }
        }

    }

    /// Given some reference to a [HistoryRegister], update the state
    /// of the folded history register in each tagged component. 
    pub fn update_history(&mut self, ghr: &HistoryRegister) {
        for comp in self.comp.iter_mut() {
            comp.csr.update(ghr);
        }
    }

}

