use std::any::Any;
use std::collections::HashSet;
use std::fmt::{self};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

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
    Nat,
    Int,
    Real,
    Complex,
    Str
}

impl InfiniteSet {
    pub fn name(&self) -> String {
        String::from(match self {
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
    pub fn intern(&mut self, set: Rc<CanonSet>) -> Rc<CanonSet> {
        if !self.pool.contains(&set) {
            self.pool.insert(Rc::clone(&set));
        }

        set
    }
}


// pub trait Set: Val + CloneSet {
//     fn is_finite(&self) -> bool;
//     fn is_countable(&self) -> bool;

//     /// Enumerates the set into values. If it cannot be enumerated, it returns [`None`].
//     fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>>;
//     fn contains(&self, other: &Box<dyn Val>) -> bool;

//     /// Checks if `self` is a subset of `other` or they're equal.
//     fn is_subset(&self, other: &Box<dyn Set>) -> bool;
// }

// pub trait CloneSet {
//     fn clone_set(&self) -> Box<dyn Set>;
// }

// impl<T> CloneSet for T
// where 
//     T: 'static + Set + Clone
// {
//     fn clone_set(&self) -> Box<dyn Set> {
//         Box::new(self.clone())
//     }
// }

// impl Clone for Box<dyn Set> {
//     fn clone(&self) -> Self {
//         self.clone_set()
//     }
// }

// /// A finite set that holds all of its elements
// /// 
// /// It saves its hash on creation, as all values are immutable so it will never change. That way it doesn't have to rehash every time it needs a hash.
// #[derive(Debug, Clone)]
// pub struct FiniteSet {
//     elements: HashSet<Box<dyn Val>>,
//     hash: u64
// }

// impl FiniteSet {
//     pub fn new(elements: HashSet<Box<dyn Val>>) -> Self {
//         let mut base = Self {
//             elements,
//             hash: 0
//         };

//         let mut state = DefaultHasher::new();
//         base.hash(&mut state);
//         base.hash = state.finish();

//         base
//     }
// }

// impl PartialEq for FiniteSet {
//     fn eq(&self, other: &Self) -> bool {
//         self.elements == other.elements
//     }
// }

// impl Hash for FiniteSet {
//     fn hash<H: Hasher>(&self, mut state: &mut H) {
//         // hash the length of the set
//         self.elements.len().hash(&mut state);

//         // Create a vector of hashes for the elements
//         let mut hashes: Vec<u64> = self.elements.iter()
//             .map(|item| {
//                 let mut hasher = DefaultHasher::new();
//                 item.hash(&mut hasher);
//                 hasher.finish()
//             })
//             .collect();

//         // Sort hashes to ensure order indpendence
//         hashes.sort_unstable();

//         // Hash the sorted hashes of the elements
//         for h in hashes {
//             h.hash(&mut state);
//         }
//     }
// }

// impl fmt::Display for FiniteSet {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{{")?;
        
//         let mut i = self.elements.len();
//         for element in self.elements.iter() {
//             if i > 1 {
//                 write!(f, "{}, ", element)?;
//             } else {
//                 write!(f, "{}", element)?;
//             }

//             i -= 1;
//         }

//         write!(f, "}}")
//     }
// }

// impl Val for FiniteSet {
//     fn compare(&self, other: &dyn Val) -> bool {
//         if let Some(other_finite_set) = other.downcast_ref::<FiniteSet>() {
//             self == other_finite_set
//         } else {
//             false
//         }
        
//     }

//     fn hash_val(&self, state: &mut dyn Hasher) {
//         state.write_u64(self.hash);
//     }

//     fn is_set(&self) -> bool {
//         true
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_boxed_any(&self) -> Box<dyn Any> {
//         Box::new(self.to_owned())
//     }

//     fn into_boxed_set(&self) -> Option<Box<dyn Set>> {
//         Some(Box::new(self.to_owned()))
//     }
// }

// impl Set for FiniteSet {
//     fn is_finite(&self) -> bool {
//         true
//     }

//     fn is_countable(&self) -> bool {
//         true
//     }

//     fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>> {
//         Some(Box::new(self.elements.iter()))
//      }

//     fn contains(&self, other: &Box<dyn Val>) -> bool {
//         self.elements.contains(other)
//     }

//     fn is_subset(&self, other: &Box<dyn Set>) -> bool {
//         if other.is_countable() {
//             self.enumerate().unwrap().all(|value| {
//                 other.contains(value)
//             })
//         } else {
//             todo!()
//         }
//     }
// }

// /* Infinite Sets

// The plan is to use predicates to determine the sets.


// -- List of Sets --
//     Whole
//     Even
//     Odd
//     Nat
//     Int
//     Real
//     Im
//     Complex

//     Bool
    
//     Char
//     Str
// */

// #[derive(Debug)]
// pub struct SetPool(HashSet<Rc<dyn Set>>);

// impl SetPool {
//     pub fn new() -> Self {
//         Self(HashSet::new())
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum InfSet {
//     Nat = 0,
//     Int,
//     Real,
//     Complex,
//     Str,
// }

// impl fmt::Display for InfSet {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl Hash for InfSet {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         state.write_u8(*self as u8);
//     }
// }

// impl Val for InfSet {
//     fn compare(&self, other: &dyn Val) -> bool {
//         if let Some(set) = other.downcast_ref::<InfSet>() {
//             match (self, set) {
//                 (a, b) if a == b => true,
//                 _ => false
//             }
//         } else {
//             false
//         }
//     }

//     fn hash_val(&self, mut state: &mut dyn Hasher) {
//         self.hash(&mut state);
//     }

//     fn is_set(&self) -> bool {
//         true
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_boxed_any(&self) -> Box<dyn Any> {
//         Box::new(self.to_owned())
//     }

//     fn into_boxed_set(&self) -> Option<Box<dyn Set>> {
//         Some(Box::new(self.to_owned()))
//     }
// }

// impl Set for InfSet {
//     fn is_finite(&self) -> bool {
//         true
//     }

//     fn is_countable(&self) -> bool {
//         match self {
//             Self::Nat   |
//             Self::Str   => true,
//             _           => false
//         }
//     }

//     fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>> {
//         todo!()
//     }

//     fn contains(&self, other: &Box<dyn Val>) -> bool {
//         match self {
//             Self::Nat => if other.is_num() {
//                 if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
//                     complex.im == BigRational::zero() && 
//                     complex.re.denom() == &BigInt::from(1) && 
//                     complex.re >= BigRational::zero()
//                 } else if let Some(bigrat) = other.downcast_ref::<BigRational>() {
//                     bigrat.denom() == &BigInt::from(1) &&
//                     bigrat >= &BigRational::zero()
//                 } else {
//                     true
//                 }
//             } else {
//                 false
//             }

//             Self::Int => if other.is_num() {
//                 if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
//                     complex.im == BigRational::zero() && 
//                     complex.re.denom() == &BigInt::from(1)
//                 } else if let Some(bigrat) = other.downcast_ref::<BigRational>() {
//                     bigrat.denom() == &BigInt::from(1)
//                 } else {
//                     true
//                 }
//             } else {
//                 false
//             }

//             Self::Real => if other.is_num() {
//                 if let Some(complex) = other.downcast_ref::<Complex<BigRational>>() {
//                     complex.im == BigRational::zero()
//                 } else {
//                     true
//                 }
//             } else {
//                 false
//             }

//             Self::Complex => other.is_num(),

//             Self::Str => other.is_str()
//         }
//     }

//     fn is_subset(&self, other: &Box<dyn Set>) -> bool {
//         todo!()
//     }
// }

// #[derive(Debug, Clone)]
// pub struct RcSet(pub Rc<dyn Set>);

// impl fmt::Display for RcSet {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.0.as_ref())
//     }
// }

// impl PartialEq for RcSet {
//     fn eq(&self, other: &Self) -> bool {
//         Rc::ptr_eq(&self.0, &other.0)
//     }
// }

// impl Hash for RcSet {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.0.as_ref().hash_val(state);
//     }
// }

// impl Val for RcSet {
//     fn compare(&self, other: &dyn Val) -> bool {
//         if let Some(other_rc) = other.downcast_ref::<RcSet>() {
//             self == other_rc
//         } else {
//             false
//         }
//     }

//     fn hash_val(&self, mut state: &mut dyn Hasher) {
//         self.hash(&mut state);
//     }

//     fn is_set(&self) -> bool {
//         true
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn as_boxed_any(&self) -> Box<dyn Any> {
//         Box::new(self.to_owned())
//     }

//     fn into_boxed_set(&self) -> Option<Box<dyn Set>> {
//         Some(Box::new(self.to_owned()))
//     }
// }

// impl Set for RcSet {
//     fn is_finite(&self) -> bool {
//         self.0.is_finite()
//     }

//     fn is_countable(&self) -> bool {
//         self.0.is_countable()
//     }

//     fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>> {
//         self.0.enumerate()
//     }

//     fn contains(&self, other: &Box<dyn Val>) -> bool {
//         self.0.contains(other)
//     }

//     fn is_subset(&self, other: &Box<dyn Set>) -> bool {
//         self.0.is_subset(other)
//     }
// }

// #[derive(Debug, Clone)]
// pub enum SetOp {
//     Union(ISet, ISet),
//     Intersect(ISet, ISet),
//     Quot(ISet, ISet),
//     SymDiff(ISet, ISet),
//     Complement(ISet)
// }

// impl SetOp {}

// //######

// /// [`ISet`] stands for "InternalSet"
// #[derive(Debug, Clone)]
// pub enum ISet {
//     Fin(FiniteSet),
//     Inf(InfSet),
//     Op(SetOp)
// }

// impl ISet {}

// impl Val for Rc<ISet> {
//     fn compare(&self, other: &dyn Val) -> bool {
//         if let Some(other_rc) = other.downcast_ref::<Rc<ISet>>() {
//             match (Rc::as_ref(self), Rc::as_ref(other_rc)) {
//                 (ISet::Fin(A), ISet::Fin(B)) => A == B,
//                 (ISet::Inf(A), ISet::Inf(B)) => A == B,
//                 (ISet::Op(A), ISet::Op(B)) => A == B,
//                 _ => false
//             }
//         } else {
//             false
//         }
//     }
// }
