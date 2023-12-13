
pub mod component;
pub use component::*;

use bitvec::prelude::*;
use crate::history::*;
use crate::direction::*;
use crate::predictor::*;
use std::ops::RangeInclusive;
use rand::prelude;


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

        // Automatically re-sort the list of components by history length. 
        // The component with the longest history length is always the 
        // first entry in the list. 
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

    pub fn predict(&self, pc: usize) -> (TAGEProvider, Outcome) { 
        for (idx, component) in self.comp.iter().enumerate() {
            if let Some(prediction) = component.try_predict(pc) {
                return (TAGEProvider::Tagged(idx), prediction);
            }
        }
        (TAGEProvider::Base, self.base.predict(pc))
    }
}

enum TAGEProvider { Base, Tagged(usize), }

impl TAGEPredictor {


    /// Find the component that will provide a prediction for the given 
    /// program counter value. 
    fn find_provider(&self, pc: usize) -> TAGEProvider {
        for (idx, component) in self.comp.iter().enumerate() {
            if component.lookup(pc).is_some() { 
                return TAGEProvider::Tagged(idx);
            }
        }
        TAGEProvider::Base
    }

    /// Update the state of all folded history registers.
    pub fn update_history(&mut self, ghr: &GlobalHistoryRegister) {
        for component in self.comp.iter_mut() {
            component.update_history(ghr);
        }
    }

    pub fn update(&mut self, 
        ghr: &GlobalHistoryRegister, 
        pc: usize, 
        outcome: Outcome
    )
    {
        match self.find_provider(pc) {
            // A hit occured in the some tagged component 'idx'
            TAGEProvider::Tagged(idx) => {
            },
            // We missed in all tagged components
            TAGEProvider::Base => {
            },
        }
        self.update_history(&ghr);
    }

}

