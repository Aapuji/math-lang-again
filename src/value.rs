use std::collections::HashSet;

use num::{BigInt, BigUint, BigRational, Complex};

use crate::types::Type;

#[derive(Debug, Clone)]
pub struct Value {
    value: Val,
    type_: Type
}

/// Representing a possible value in the language.
#[derive(Debug, Clone)]
pub enum Val {
    Uint(BigUint),
    Int(BigInt),
    Rational(BigRational),
    Complex(Complex<BigRational>),
    Ascii(u8),
    Char(char),
    String(String),
    Bool(bool),
    FiniteSet(HashSet<Val>)
    // Others todo
}

// Todo, values with different Vals (like Uint 100 vs Int 100) are the same value, but are different in internals. So hashing will be different, so there has to be a preliminary step.
