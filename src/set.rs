use std::any::Any;
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use num::bigint::{self, Sign};
use num::{BigInt, BigRational, Complex, Zero};

use crate::iter::ValIterator;
use crate::value::Val;

pub trait Set {
    fn is_finite(&self) -> bool;
    fn is_countable(&self) -> bool;

    /// Enumerates the set into values. If it cannot be enumerated, it returns [`None`].
    fn enumerate<I: ValIterator>(&self) -> Option<I>;
    fn contains(&self, other: &Box<dyn Val>) -> bool;

    /// Checks if `self` is a subset of `other` or they're equal.
    fn is_subset(&self, other: &Rc<CanonSet>) -> bool;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CanonSet {
    Finite(FiniteSet),
    Infinite(InfiniteSet),
    Union(Rc<Self>, Rc<Self>),
    Intersect(Rc<Self>, Rc<Self>),
    SymDiff(Rc<Self>, Rc<Self>),
    Exclusion(Rc<Self>, Rc<Self>),
    Complement(Rc<Self>)
}

impl fmt::Display for CanonSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Finite(set) => write!(f, "{}", set),
            Self::Infinite(set) => write!(f, "{}", set),
            Self::Union(a, b) => write!(f, "{} | {}", a, b),
            Self::Intersect(a, b) => write!(f, "{} & {}", a, b),
            Self::SymDiff(a, b) => write!(f, "{} ~ {}", a, b),
            Self::Exclusion(a, b) => write!(f, "{} \\ {}", a, b),
            Self::Complement(set) => write!(f, "~{}", set)
        }
    }
}

/// Logic to canonicalize the set expression tree
pub fn canon(set: Rc<CanonSet>) -> Rc<CanonSet> {
    // placeholder for now
    set
}

impl Val for Rc<CanonSet> {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_set) = other.downcast_ref::<Rc<CanonSet>>() {
            self == other_set
        } else {
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }

    fn is_set(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
    }
}

impl Set for CanonSet {
    fn is_finite(&self) -> bool {
        match self {
            Self::Finite(set) => set.is_finite(), 
            Self::Infinite(set) => set.is_finite(),

            _ => todo!()
        }
    }

    fn is_countable(&self) -> bool {
        match self {
            Self::Finite(set) => set.is_countable(),
            Self::Infinite(set) => set.is_countable(),

            _ => todo!()
        }
    }

    fn enumerate<I: ValIterator>(&self) -> Option<I> {
        todo!()
    }

    fn contains(&self, other: &Box<dyn Val>) -> bool {
        match self {
            Self::Finite(set) => set.contains(other),
            Self::Infinite(set) => set.contains(other),

            _ => todo!()
        }
    }

    fn is_subset(&self, other: &Rc<Self>) -> bool {
        match self {
            _ => todo!()
        }
    }
}

/// A finite set that holds all of its elements
/// 
/// It saves its hash on creation, as all values are immutable so it will never change. That way it doesn't have to rehash every time it needs a hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteSet {
    elements: HashSet<Box<dyn Val>>,
    hash: u64
}

impl FiniteSet {
    pub fn new(elements: HashSet<Box<dyn Val>>) -> Self {
        let mut base = Self {
            elements,
            hash: 0
        };

        let mut state = DefaultHasher::new();
        base.hash(&mut state);
        base.hash = state.finish();

        base
    }
}

impl Hash for FiniteSet {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        // hash the length of the set
        self.elements.len().hash(&mut state);

        // Create a vector of hashes for the elements
        let mut hashes: Vec<u64> = self.elements.iter()
            .map(|item| {
                let mut hasher = DefaultHasher::new();
                item.hash(&mut hasher);
                hasher.finish()
            })
            .collect();

        // Sort hashes to ensure order indpendence
        hashes.sort_unstable();

        // Hash the sorted hashes of the elements
        for h in hashes {
            h.hash(&mut state);
        }
    }
}

impl fmt::Display for FiniteSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        
        let mut i = self.elements.len();
        for element in self.elements.iter() {
            if i > 1 {
                write!(f, "{}, ", element)?;
            } else {
                write!(f, "{}", element)?;
            }

            i -= 1;
        }

        write!(f, "}}")
    }
}

impl Set for FiniteSet {
    fn is_finite(&self) -> bool {
        true
    }

    fn is_countable(&self) -> bool {
        true
    }

    fn enumerate<I: ValIterator>(&self) -> Option<I> {
        todo!()
    }

    fn contains(&self, other: &Box<dyn Val>) -> bool {
        self.elements.contains(other)
    }

    fn is_subset(&self, other: &Rc<CanonSet>) -> bool {
        match other.as_ref() {
            CanonSet::Finite(set) => self == set,
            
            _ => todo!()
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InfiniteSet {
    Univ,
    Nat,
    Int,
    Real,
    Complex,
    Str
}

impl InfiniteSet {
    pub fn name(&self) -> String {
        String::from(match self {
            Self::Univ => "Univ",
            Self::Nat => "Nat",
            Self::Int => "Int",
            Self::Real => "Real",
            Self::Complex => "Complex",
            Self::Str => "Str"
        })
    }
}

impl fmt::Display for InfiniteSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Set for InfiniteSet {
    fn is_finite(&self) -> bool {
        false
    }

    fn is_countable(&self) -> bool {
        match self {
            Self::Nat |
            Self::Int |
            Self::Str => true,

            _ => false
        }
    }

    fn enumerate<I: ValIterator>(&self) -> Option<I> {
        match self {
            _ => todo!()
        }
    }

    fn contains(&self, other: &Box<dyn Val>) -> bool {
        match self {
            Self::Univ => true,
            Self::Nat => if other.is_num() {
                if let Some(bigint) = other.downcast_ref::<BigInt>() {
                    bigint.sign() != Sign::Minus
                } else if let Some(bigrat) = other.downcast_ref::<BigRational>() {
                    bigrat.is_integer() && bigrat.numer().sign() != Sign::Minus
                } else if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
                    complex.im == BigRational::zero() && complex.re.is_integer() && complex.re.numer().sign() != Sign::Minus
                } else if let Some(_) = other.downcast_ref::<bool>() {
                    true
                } else {
                    false
                }
            } else {
                false
            }

            Self::Int => if other.is_num() {
                if let Some(_) = other.downcast_ref::<BigInt>() {
                    true
                } else if let Some(bigrat) = other.downcast_ref::<BigRational>() {
                    bigrat.is_integer()
                } else if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
                    complex.im == BigRational::zero() && complex.re.is_integer()
                } else if let Some(_) = other.downcast_ref::<bool>() {
                    true
                } else {
                    false
                }
            } else {
                false
            }

            Self::Real => if other.is_num() {
                if let Some(_) = other.downcast_ref::<BigInt>() {
                    true
                } else if let Some(_) = other.downcast_ref::<BigRational>() {
                    true
                } else if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
                    complex.im == BigRational::zero()
                } else if let Some(_) = other.downcast_ref::<bool>() {
                    true
                } else {
                    false
                }
            } else {
                false
            }

            Self::Complex => other.is_num(), // as of now, Complex is the all-encompassing numeric type. Perhaps in future this will be changed. Perhaps a Num class or smth. Also, there may be other number types as well, like Alg, Even, Odd, etc.

            Self::Str => if other.is_str() {
                if let Some(_) = other.downcast_ref::<String>() {
                    true
                } else {
                    false
                }
            } else {
                // todo, allow for casting to string
                // perhapse, can_str for can be casted to str
                false
            }
            _ => todo!()
        }
    }

    fn is_subset(&self, other: &Rc<CanonSet>) -> bool {
        todo!()
    }


}

#[derive(Debug)]
pub struct SetPool {
    pool: HashSet<Rc<CanonSet>>
}

impl SetPool {
    pub fn new() -> Self {
        SetPool {
            pool: HashSet::new()
        }
    }

    /// Interns the given [`Rc<Set>`] and returns it back out. If it is new, it will intern it to the [`SetPool`], otherwise it will just return it
    pub fn intern(&mut self, set: &Rc<CanonSet>) -> Rc<CanonSet> {
        if !self.pool.contains(set) {
            self.pool.insert(Rc::clone(set));
        }

        Rc::clone(set)
    }
}
