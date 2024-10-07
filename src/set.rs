use std::any::Any;
use std::collections::HashSet;
use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::value::Val;

pub trait Set: Val + CloneSet {
    fn is_finite(&self) -> bool;
    fn is_countable(&self) -> bool;

    /// Enumerates the set into values. If it cannot be enumerated, it returns [`None`].
    fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>>;
    fn contains(&self, other: &Box<dyn Val>) -> bool;

    /// Checks if `self` is a subset of `other` or they're equal.
    fn is_subset(&self, other: &Box<dyn Set>) -> bool;
}

pub trait CloneSet {
    fn clone_set(&self) -> Box<dyn Set>;
}

impl<T> CloneSet for T
where 
    T: 'static + Set + Clone
{
    fn clone_set(&self) -> Box<dyn Set> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Set> {
    fn clone(&self) -> Self {
        self.clone_set()
    }
}

/// A finite set that holds all of its elements
/// 
/// It saves its hash on creation, as all values are immutable so it will never change. That way it doesn't have to rehash every time it needs a hash.
#[derive(Debug, Clone)]
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

impl PartialEq for FiniteSet {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
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

impl Val for FiniteSet {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_finite_set) = other.downcast_ref::<FiniteSet>() {
            self == other_finite_set
        } else {
            false
        }
        
    }

    fn hash_val(&self, state: &mut dyn Hasher) {
        state.write_u64(self.hash);
    }

    fn is_mat(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
    }
}

impl Set for FiniteSet {
    fn is_finite(&self) -> bool {
        true
    }

    fn is_countable(&self) -> bool {
        true
    }

    fn enumerate(&self) -> Option<Box<dyn Iterator<Item = &Box<dyn Val>> + '_>> {
        Some(Box::new(self.elements.iter()))
     }

    fn contains(&self, other: &Box<dyn Val>) -> bool {
        self.elements.contains(other)
    }

    fn is_subset(&self, other: &Box<dyn Set>) -> bool {
        if other.is_countable() {
            self.enumerate().unwrap().all(|value| {
                other.contains(value)
            })
        } else {
            todo!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct InfiniteStruct {

    hash: u64
}


