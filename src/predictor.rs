
use crate::direction::*;

pub trait Predictor {
    fn predict(&self) -> Outcome;
    fn update(&mut self);
}


