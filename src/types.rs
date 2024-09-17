
//! Note: I'm calling "type-set demotion" when a type is interpreted as a set. I'm calling "set-type promotion" when a set is interpreted as a type.

use std::collections::HashSet;

/// A builtin type in the language, which can demote to an infinite set.
/// 
/// <hr>
/// 
/// All userland types will be a combination of these types, set operations, collections, and conditionals/checking.
/// 
/// For example a Point can be represented:
/// ```
/// data Point(N) = (Int; N)
/// ```
/// This `Point` is a collection (tuple) of a builtin type.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // All-Encompassing
    Univ, Empty, 
    Type,
    
    // Numeric
    Whole, Nat, Int, Rat, Alg, Real, Complex,
    Even, Odd, Zero,

    // Text
    Ascii, Char, Grapheme, Str,

    // Boolean
    Bool,
    Bit,

    // Function
    Func(Box<[Type]>, Box<Type>),

    // Collection
    Set(Box<Type>), 
    Tup(Box<[Type]>), List(Box<Type>, usize), 
    Vec(Box<Type>, usize), Mat(Box<Type>, usize, usize), Tensor(Box<Type>, Box<[usize]>),
    Record(Box<Type>, Box<Type>),

    // Combination
    Union(Box<Type>, Box<Type>),        // A | B
    Intersect(Box<Type>, Box<Type>),    // A & B
    Diff(Box<Type>, Box<Type>),         // A \ B
    Not(Box<Type>),                     // ~A

    // Atom
    Atom(String),

    // Set of Items
    ValueSet(FiniteSet) // like {1, 2, 3} or {"hi", true, (0, 0)}
}

impl Type {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

// Placeholder
type FiniteSet = HashSet<u8>;
