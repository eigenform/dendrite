
use crate::Outcome;
use crate::predictor::SimplePredictor;

/// A simple predictor with no state: randomly predict an outcome.
pub struct RandomPredictor;
impl SimplePredictor for RandomPredictor {
    fn name(&self) -> &'static str { "RandomPredictor" }
    fn predict(&self) -> Outcome { rand::random::<bool>().into() }
}

/// A simple predictor with no state: always predict 'taken'.
pub struct TakenPredictor;
impl SimplePredictor for TakenPredictor {
    fn name(&self) -> &'static str { "TakenPredictor" }
    fn predict(&self) -> Outcome { Outcome::T }
}

/// A simple predictor with no state: always predict 'not-taken'.
pub struct NotTakenPredictor;
impl SimplePredictor for NotTakenPredictor {
    fn name(&self) -> &'static str { "NotTakenPredictor" }
    fn predict(&self) -> Outcome { Outcome::N }
}

