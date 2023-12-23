
pub mod component;
pub use component::*;

use bitvec::prelude::*;
use rand::distributions::{ WeightedIndex, Distribution };

use crate::history::*;
use crate::Outcome;
use crate::predictor::*;


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

    /// A predicted direction
    pub outcome: Outcome,

    /// Alternate component used to provide a prediction
    pub alt_provider: TAGEProvider,

    /// Predicted direct from the alternate component
    pub alt_outcome: Outcome,

    /// The index identifying the entry used to make this prediction
    pub idx: usize,

    /// The tag matching the entry used to make this prediction
    pub tag: usize,
}

pub struct TAGEConfig {
    pub base: TAGEBaseConfig,
    pub comp: Vec<TAGEComponentConfig>,
}
impl TAGEConfig {
    pub fn new(base: TAGEBaseConfig) -> Self {
        Self {
            base,
            comp: Vec::new(),
        }
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
        let comp = self.comp.iter().map(|c| c.clone().build()).collect();
        let base = self.base.build();
        TAGEPredictor {
            cfg: self,
            base,
            comp,
        }
    }
}



/// The "TAgged GEometric history length" predictor. 
///
/// See "A case for (partially) TAgged GEometric history length branch 
/// prediction" (Seznec, 2006).
pub struct TAGEPredictor { 
    pub cfg: TAGEConfig,

    /// Base component
    pub base: TAGEBaseComponent,

    /// Tagged components
    pub comp: Vec<TAGEComponent>,
}
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
            let tag = component.get_tag(pc);
            tags.push(tag);
        }
        tags
    }

    pub fn predict(&self, pc: usize) -> TAGEPrediction {
        let (base, tagged) = self.get_all_entries(pc);
        let tags = self.get_all_tags(pc);

        let mut result = TAGEPrediction {
            provider: TAGEProvider::Base,
            outcome: base.predict(),
            alt_provider: TAGEProvider::Base,
            alt_outcome: base.predict(),
            idx: self.base.get_index(pc),
            tag: 0,
        };

        // NOTE: You're iterating through components *backwards* here 
        // (from the shortest to longest history length).
        let tagged_iter = tagged.iter().enumerate().zip(tags.iter()).rev();
        for ((idx, entry), tag) in tagged_iter { 
            let hit = if let Some(v) = entry.tag { v == *tag } else { false };
            if hit { 
                result.alt_provider = result.provider;
                result.alt_outcome = result.outcome;
                result.provider = TAGEProvider::Tagged(idx);
                result.outcome = entry.predict();
                result.idx = self.comp[idx].get_index(pc);
                result.tag = *tag; 
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

        // Update the entry in the component that provided the prediction
        match prediction.provider {
            TAGEProvider::Base => {
                let entry = self.base.get_entry_mut(pc);
                entry.update(outcome);
            },
            TAGEProvider::Tagged(idx) => {
                let entry = self.comp[idx].get_entry_mut(pc);
                entry.ctr.update(outcome);
                entry.decrement_useful();
            },
        }

        // Try to allocate a new entry: 
        if let Some(idx) = self.select_alloc_candidate(pc, prediction.provider) {
            //println!("[*] Allocated in comp{}", idx);
            let new_tag   = self.comp[idx].get_tag(pc);
            let new_entry = self.comp[idx].get_entry_mut(pc);
            new_entry.invalidate();
            new_entry.tag = Some(new_tag);
            new_entry.useful = 0;
            new_entry.ctr.set_direction(outcome);
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
        // Update the entry in the component that provided the prediction
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

    /// Given a particular prediction and the resolved outcome, update the 
    /// state of the predictor. 
    ///
    /// NOTE: The paper suggests periodically resetting the 'useful' bits
    /// on all counters (ie. every 256,000 branches?)
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

    /// Given some reference to a [GlobalHistoryRegister], update the state
    /// of the folded history register in each tagged component. 
    pub fn update_history(&mut self, ghr: &GlobalHistoryRegister) {
        for comp in self.comp.iter_mut() {
            comp.csr.update(ghr);
        }
    }

}


