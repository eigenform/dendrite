
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
    provider: TAGEProvider,
    /// Alternate component used to provide a prediction
    alt_provider: TAGEProvider,
    /// A predicted direction
    outcome: Outcome,
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

    /// Given the *non-zero* index of a tagged component that provided an
    /// incorrect prediction, select a tagged component with a longer 
    /// associated history length that will be used to allocate a new entry.
    pub fn select_alloc_candidate(&self, pc: usize, provider_idx: usize) 
        -> Option<usize>
    { 
        assert!(provider_idx != 0);

        let mut candidates: Vec<usize> = Vec::new();

        for (idx, component) in self.comp.iter().enumerate() {
            // Component must have a longer history length
            if idx >= provider_idx { continue; }
            // The entry we're replacing must have the useful bits cleared
            let entry = component.get_entry(pc);
            if entry.useful == 0 {
                candidates.push(idx);
            }
        }

        // We failed allocate a new entry
        if candidates.is_empty() {
            return None;
        }

        // Easy case: there's only a single candidate 
        if candidates.len() == 1 {
            return candidates.first().copied();
        }

        // Otherwise, select between multiple candidates where the probability 
        // scales *down* with candidates of increasing history length. 
        // Given candidates with history lengths J and K [where J < K], the 
        // candidate J is twice as likely to be chosen over K.
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

    pub fn update(&mut self, 
        pc: usize, 
        prediction: TAGEPrediction,
        outcome: Outcome
    )
    {
        let misprediction = prediction.outcome != outcome;
        let alloc = match prediction.provider {
            TAGEProvider::Base => {
            },
            TAGEProvider::Tagged(idx) => {
            },
        };
    }

}




