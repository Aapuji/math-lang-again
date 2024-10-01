use std::any::Any;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use num::{BigInt, BigRational, Complex};

pub trait Val: Any + Debug {
    fn compare(&self, other: &dyn Val) -> bool;
    fn hash_val(&self, state: &mut dyn Hasher);
    fn as_any(&self) -> &dyn Any;
}

impl dyn Val {
    fn downcast_ref<T: Val>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl PartialEq for dyn Val {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other)
    }
}

impl Eq for dyn Val {}

impl Hash for dyn Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_val(state);
    }
}

impl Val for BigInt {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_int) = other.downcast_ref::<BigInt>() {
            // Compare two BigInts directly
            self == other_int
        } else if let Some(other_rational) = other.downcast_ref::<BigRational>() {
            // Compare BigInt with BigRational by converting BigInt to BigRational
            &BigRational::from(self.clone()) == other_rational
        } else if let Some(other_complex) = other.downcast_ref::<Complex<BigRational>>() {
            // Compare BigInt with Complex (where im must be 0)
            if other_complex.im != BigRational::from(BigInt::from(0)) {
                false
            } else {
                let real_part = BigRational::from(self.clone());
                other_complex.re == real_part
            }
        } else {
            // Comparison with non-numeric types, liek strings, is false
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        Complex::<BigRational>::from(BigRational::from(self.clone())).hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Val for BigRational {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_real) = other.downcast_ref::<BigRational>() {
            self == other_real
        } else if let Some(other_int) = other.downcast_ref::<BigInt>() {
            self == &BigRational::from(other_int.clone())
        } else if let Some(other_complex) = other.downcast_ref::<Complex<BigRational>>() {
            other_complex.im == BigRational::from_integer(BigInt::from(0)) && other_complex.re == *self
        } else {
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        Complex::<BigRational>::from(self.clone()).hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Val for Complex<BigRational> {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_complex) = other.downcast_ref::<Complex<BigRational>>() {
            self == other_complex
        } else if let Some(other_real) = other.downcast_ref::<BigRational>() {
            self.im == BigRational::from_integer(BigInt::from(0)) && self.re == *other_real
        } else if let Some(other_int) = other.downcast_ref::<BigInt>() {
            self.im == BigRational::from_integer(BigInt::from(0)) && self.re == BigRational::from(other_int.clone())
        } else {
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Val for String {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_str) = other.downcast_ref::<String>() {
            self == other_str
        } else {
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait Set: Val {
    fn is_finite(&self) -> bool;
    fn is_countable(&self) -> bool;
}

/// A finite set that holds all of its elements
/// 
/// It saves its hash on creation, as all values are immutable so it will never change. That way it doesn't have to rehash every time it needs a hash.
#[derive(Debug)]
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

        // todo, add some extra stuff so that you can't just emulate it 
        // right now it hashes length, then values, then sorts
        // so yuo can emulate {{0, 1, 2}} as {3, 0, 1, 2}, and they should collide.
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Set for FiniteSet {
    fn is_finite(&self) -> bool {
        true
    }

    fn is_countable(&self) -> bool {
        true
    }
}





// Represents the internal representation of a value
// #[derive(Debug, Clone)]
// pub enum Val {
//     Int(BigInt),
//     Decimal(BigRational),
//     Complex(Complex<BigRational>),
//     String(String),
//     Symbol(String) // perhaps have a table to map an ident to a number and then use that number to refer it??
// }

// Todo, values with different Vals (like Uint 100 vs Int 100) are the same value, but are different in internals. So hashing will be different, so there has to be a preliminary step.
