//! Implementations of different branch predictors. 

pub mod table;
pub mod simple;
pub mod tage;
pub mod gshare; 
pub mod pht;
pub mod counter; 
pub mod perceptron;
pub mod btb; 

pub use table::*;
pub use simple::*;
pub use counter::*;
pub use perceptron::*;
pub use tage::*;
pub use btb::*;

use crate::history::*;
use crate::Outcome;


/// Interface to a "trivial" predictor that guesses an outcome without 
/// accepting feedback from the rest of the machine. 
pub trait SimplePredictor {
    fn name(&self) -> &'static str;
    fn predict(&self) -> Outcome;
}

/// Interface to a predictor with some internal state which is only subject to 
/// change by the correct branch outcome.
pub trait StatefulPredictor { 
    fn name(&self) -> &'static str;

    /// Reset the internal state of the predictor.
    fn reset(&mut self);

    /// Return the current predicted outcome.
    fn predict(&self) -> Outcome;

    /// Update the internal state of the predictor with the correct outcome.
    fn update(&mut self, outcome: Outcome);
}

/// Implemented on types that represent some *configuration* for a predictor.
pub trait PredictorConfiguration {
    /// The type of object that will be created using this configuration.
    type Predictor; 

    /// Return the [approximate] number of storage bits used to implement 
    /// this predictor. 
    fn storage_bits(&self) -> usize; 

    /// Use this configuration to build an instance of the predictor.
    fn build(self) -> Self::Predictor;
}


pub enum InputState {
    ProgramCounter(usize),
}




