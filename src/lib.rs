
pub mod history;
pub mod target; 
pub mod direction;
pub mod predictor;
pub mod tage;

pub mod sim;

pub use history::*;
pub use target::*;
pub use direction::*;

pub use predictor::*;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgramCounter(pub usize);


