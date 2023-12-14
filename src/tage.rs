
pub mod component;
pub use component::*;

use bitvec::prelude::*;
use crate::history::*;
use crate::direction::*;
use crate::predictor::*;
use std::ops::RangeInclusive;
use rand::prelude;
use rand::distributions::{ WeightedIndex, Distribution };

/// Identifies a particular TAGE component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TAGEProvider { 
    /// The base component
    Base, 
    /// A tagged component
    Tagged(usize), 
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TAGEPrediction {
    /// The component providing the prediction
    pub provider: TAGEProvider,
    /// Alternate component used to provide a prediction
    pub alt_provider: TAGEProvider,
    /// A predicted direction
    pub outcome: Outcome,
}

/// The "TAgged GEometric history length" predictor. 
///
/// See "A case for (partially) TAgged GEometric history length branch 
/// prediction" (Seznec, 2006).
///
pub struct TAGEPredictor { 
    /// Base component
    pub base: TAGEBaseComponent,

    /// Tagged components
    pub comp: Vec<TAGEComponent>,
}
impl TAGEPredictor {
    /// Create a new predictor with some base component. The user is expected 
    /// to add tagged components with [TAGEPredictor::add_component].
    pub fn new(base: TAGEBaseComponent) -> Self {
        Self { 
            base, comp: Vec::new()
        }
    }

    /// Add a tagged component to the predictor.
    pub fn add_component(&mut self, component: TAGEComponent) {
        self.comp.push(component);

        // When adding a new component, automatically re-sort the list of 
        // components by history length. This way, the component with the 
        // longest history length is always guaranteed to be the first entry 
        // in the list. 
        self.comp.sort_by(|x, y| {
            let x_history_len = x.ghr_range.end() - x.ghr_range.start();
            let y_history_len = y.ghr_range.end() - y.ghr_range.start();
            std::cmp::Ord::cmp(&y_history_len, &x_history_len)
        });
    }

    /// Return the number of tagged components.
    pub fn num_tagged_components(&self) -> usize { 
        self.comp.len()
    }

    /// Return the index of the tagged component with the shortest associated
    /// history length.
    pub fn shortest_tagged_component(&self) -> usize { 
        self.num_tagged_components() - 1
    }

    /// Given a program counter value and the provider of an incorrect 
    /// prediction, try to select a tagged component that will be used to 
    /// allocate a new entry. 
    fn select_alloc_candidate(&self, pc: usize, provider: TAGEProvider) 
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
            let entry = self.comp[idx].get_entry(pc);
            if entry.useful == 0 {
                candidates.push(idx);
            }
        }

        // Easy case: we failed allocate a new entry
        if candidates.is_empty() {
            return None;
        }

        // Easy case: there's only a single candidate 
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

    /// Index into all components and return references to all entries that 
    /// correspond to the given program counter value.
    pub fn get_all_entries(&self, pc: usize) 
        -> (&SaturatingCounter, Vec<&TAGEEntry>) 
    {
        let mut entries = Vec::new();
        for component in self.comp.iter() {
            entries.push(component.get_entry(pc));
        }
        (self.base.get_entry(pc), entries)
    }

    pub fn get_all_tags(&self, pc: usize) -> Vec<usize> { 
        let mut tags = Vec::new();
        for component in self.comp.iter() {
            tags.push(0);
        }
        tags
    }

    pub fn predict(&self, pc: usize) -> TAGEPrediction {
        let (base, tagged) = self.get_all_entries(pc);
        let tags = self.get_all_tags(pc);
        let tagged_iter = tagged.iter().enumerate().zip(tags.iter());

        let mut result = TAGEPrediction {
            provider: TAGEProvider::Base,
            alt_provider: TAGEProvider::Base,
            outcome: base.predict(),
        };

        for ((idx, entry), tag) in tagged_iter { 
            let hit = if let Some(v) = entry.tag { v == *tag } else { false };
            if hit { 
                result.alt_provider = result.provider;
                result.provider = TAGEProvider::Tagged(idx);
                result.outcome = entry.predict();
            }
        }
        result
    }

    fn update_on_misprediction(&mut self, 
        pc: usize, 
        prediction: TAGEPrediction, 
        outcome: Outcome
    )
    {
        // Try to allocate a new entry
        if let Some(idx) = self.select_alloc_candidate(pc, prediction.provider) {
            println!("[*] Allocated in comp{}", idx);
            let new_entry = self.comp[idx].get_entry_mut(pc);
            new_entry.useful = 1;
            new_entry.tag = Some(0);
            new_entry.ctr.update(outcome);
        } 
        // Otherwise, use some strategy to age all of the entries
        else { 
        }

    }

    fn update_on_correct_prediction(&mut self,
        pc: usize,
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        match prediction.provider {
            TAGEProvider::Base => {
                let entry = self.base.get_entry_mut(pc);
                entry.update(outcome);
            },
            TAGEProvider::Tagged(idx) => {
                let entry = self.comp[idx].get_entry_mut(pc);
                entry.increment_useful();
                entry.ctr.update(outcome);
            },
        }
    }

    pub fn update(&mut self, 
        pc: usize, 
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        let misprediction = prediction.outcome != outcome;
        if misprediction {
            self.update_on_misprediction(pc, prediction, outcome);
        } 
        else {
            self.update_on_correct_prediction(pc, prediction, outcome);
        }

    }

    pub fn update_history(&mut self, ghr: &GlobalHistoryRegister) {
        for comp in self.comp.iter_mut() {
            comp.csr.update(ghr);
        }
    }

}




