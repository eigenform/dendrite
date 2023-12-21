#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(unused_mut)]

pub mod history;
pub mod predictor;
pub mod trace;

pub use trace::*;
pub use history::*;
pub use predictor::*;

/// A branch outcome. 
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { N = 0, T = 1 }
impl std::ops::Not for Outcome { 
    type Output = Self;
    fn not(self) -> Self { 
        match self { 
            Self::N => Self::T,
            Self::T => Self::N,
        }
    }
}
impl From<bool> for Outcome {
    fn from(x: bool) -> Self { 
        match x { 
            true => Self::T,
            false => Self::N 
        }
    }
}
impl Into<bool> for Outcome {
    fn into(self) -> bool { 
        match self { 
            Self::T => true,
            Self::N => false,
        }
    }
}

