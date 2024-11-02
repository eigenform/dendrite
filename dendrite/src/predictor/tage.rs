//! Implementations of a "Tagged GEometric history length" (TAGE) predictor. 

pub mod component;
pub mod stat;
pub mod config;

pub use component::*;
pub use stat::*;
pub use config::*;

use bitvec::prelude::*;
use rand::distributions::{ WeightedIndex, Distribution };

use crate::history::*;
use crate::Outcome;
use crate::predictor::*;

/// Container for inputs passed to a [`TAGEPredictor`] and its components.
#[derive(Clone)]
pub struct TAGEInputs {
    /// Program counter associated with a predicted branch
    pub pc: usize,

    ///// Bits from a path history register
    //pub phr: &'a HistoryRegister,
}


/// Identifies a particular component in a [`TAGEPredictor`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TAGEProvider { 
    /// The base component
    Base, 

    /// A tagged component
    Tagged(usize), 
}

/// Container for output from [`TAGEPredictor::predict`], including the 
/// predicted outcome and other metadata about how the prediction was made.
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


/// The "TAgged GEometric history length" predictor. 
///
/// See the following: 
///  - "A case for (partially) TAgged GEometric history length branch prediction" 
///  (Seznec, 2006).
pub struct TAGEPredictor { 
    /// The configuration used to create this object
    pub cfg: TAGEConfig,

    pub stat: TAGEStats,

    /// Base component
    pub base: TAGEBaseComponent,

    /// Tagged components
    pub comp: Vec<TAGEComponent>,

    /// Counter used to periodically reset all 'useful' counters
    pub reset_ctr: u8,
}
impl TAGEPredictor {

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

    /// Given a program counter value and the provider of an incorrect 
    /// prediction, try to select a tagged component that will be used to 
    /// allocate a new entry. 
    ///
    /// Returns [None] if we fail to allocate a new entry. 
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
        //
        // NOTE: In hardware, this is presumably just an LFSR
        let mut rng = rand::thread_rng();
        let weights: Vec<usize> = candidates.iter().map(|idx| 1 << idx)
            .collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        Some(candidates[dist.sample(&mut rng)])
    }

    /// Update the predictor to account for a misprediction. 
    fn update_incorrect(&mut self, 
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

                self.stat.base_miss += 1;
            },
            TAGEProvider::Tagged(idx) => {
                let index = self.comp[idx].get_index(input.clone());
                let entry = self.comp[idx].get_entry_mut(index);
                entry.ctr.update(outcome);
                //entry.decrement_useful();

                self.stat.comp_miss[idx] += 1;
            },
        }

        // Try to allocate a new entry. 
        // If we've succeeded, initialize the new entry with the correct
        // outcome [in the weakest state] and reset the 'useful' counter.
        //
        // All allocation attempts are tracked with an 8-bit counter which is 
        // incremented on failure and decremented on success. 
        // When this counter saturates, we reset the state of all 'useful'
        // counters in an attempt to free up some entries. 

        if let Some(idx) = self.alloc(input.clone(), prediction.provider) {
            let new_index = self.comp[idx].get_index(input.clone());
            let new_tag   = self.comp[idx].get_tag(input.clone());
            let new_entry = self.comp[idx].get_entry_mut(new_index);
            new_entry.invalidate(self.stat.clk);
            new_entry.tag = Some(new_tag);
            new_entry.useful = 0;
            new_entry.ctr.set_direction(outcome);
            new_entry.ctr.set_strength(0);

            new_entry.stat.branches.insert(input.pc);
            self.stat.alcs += 1;
            self.reset_ctr = self.reset_ctr.saturating_add(1);
        } 
        else { 
            self.stat.failed_alcs += 1;
            self.reset_ctr = self.reset_ctr.saturating_sub(1);
        }

    }

    /// Update the predictor to account for a correct prediction.
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
                self.stat.base_hits += 1;
                entry.update(outcome);
            },

            // Increment when the alternate prediction is incorrect
            TAGEProvider::Tagged(idx) => {
                let index = self.comp[idx].get_index(input.clone());
                let entry = self.comp[idx].get_entry_mut(index);

                if prediction.alt_outcome != outcome {
                    entry.increment_useful();
                }

                self.stat.comp_hits[idx] += 1;
                entry.ctr.update(outcome);
            },
        }
    }

}

/// The public interface to a [`TAGEPredictor`].
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

        // Access the base component and all tagged components 
        let (base_idx, base_entry) = self.get_base_entry(input.clone());
        let tagged_entries = self.get_tagged_entries(input.clone());

        // The base component provides the default predicted outcome 
        // for cases where we miss in all tagged components
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

        // Find the longest-length tagged component that yields a match
        let tagged_iter = tagged_entries.iter().enumerate();
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
                break;
            }
        }
        result
    }

    /// Given a particular prediction and the resolved outcome, update the 
    /// state of the predictor. 
    pub fn update(&mut self, 
        input: TAGEInputs, 
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        if prediction.outcome != outcome {
            self.update_incorrect(input.clone(), prediction, outcome);
        } else {
            self.update_correct(input.clone(), prediction, outcome);
        }

        // Periodically reset *all* of the 'useful' counters across all 
        // tagged components. 
        if self.reset_ctr == u8::MAX {
            self.reset_ctr = 0;
            self.stat.resets += 1;
            for comp in self.comp.iter_mut() {
                comp.reset_useful_bits();
            }
        }

        self.stat.clk += 1;
    }

    /// Given some reference to a [`HistoryRegister`], update the state
    /// of the folded history register in each tagged component. 
    pub fn update_history(&mut self, ghr: &HistoryRegister) {
        for comp in self.comp.iter_mut() {
            comp.csr.update(ghr);
        }
    }

}

