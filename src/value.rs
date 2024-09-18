
use num::{BigInt, BigRational};

use crate::types::Type;

#[derive(Debug, Clone)]
pub struct Value {
    value: Val,
    type_: Type
}

impl Value {
    pub fn new(value: Val, type_: Type) -> Self {
        Self {
            value,
            type_
        }
    }
}

/// Represents the internal representation of a value
#[derive(Debug, Clone)]
pub enum Val {
    Int(BigInt),
    Decimal(BigRational),
}

// Todo, values with different Vals (like Uint 100 vs Int 100) are the same value, but are different in internals. So hashing will be different, so there has to be a preliminary step.
