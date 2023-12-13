
pub mod component;
pub use component::*;

use bitvec::prelude::*;
use crate::history::*;
use crate::direction::*;
use crate::predictor::*;
use std::ops::RangeInclusive;
use rand::prelude;

/// Identifies a particular TAGE component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TAGEProvider { 
    /// The base component
    Base, 
    /// A tagged component
    Tagged(usize), 
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

    pub fn predict(&self, pc: usize) -> (TAGEProvider, Outcome) {
        // Index into all tables
        let (base, tagged) = self.get_all_entries(pc);

        let tags = self.get_all_tags(pc);
        let hit = tagged.iter().enumerate().zip(tags.iter()).find(
            |((idx, entry), tag)| { 
                if let Some(v) = entry.tag { v == **tag } else { false }
            }
        );

        if let Some(((idx, entry), tag)) = hit { 
            return (TAGEProvider::Tagged(idx), entry.predict());
        }
        (TAGEProvider::Base, base.predict())
    }

    pub fn update(&mut self, 
        pc: usize, 
        provider: TAGEProvider, 
        outcome: Outcome
    )
    {
        match provider { 
            TAGEProvider::Base => {
            },
            TAGEProvider::Tagged(idx) => { 
            },
        }
    }

}




