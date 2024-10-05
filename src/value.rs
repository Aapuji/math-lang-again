use std::any::Any;
use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use std::hash::{DefaultHasher, Hash, Hasher};
use num::{BigInt, BigRational, Complex};

pub trait Val: Any + Debug + Display {
    fn compare(&self, other: &dyn Val) -> bool;
    fn hash_val(&self, state: &mut dyn Hasher);

    fn is_num(&self) -> bool { false }
    fn is_str(&self) -> bool { false }
    fn is_tup(&self) -> bool { false }
    fn is_mat(&self) -> bool { false }
    fn is_set(&self) -> bool { false }

    fn as_any(&self) -> &dyn Any;
    fn as_boxed_any(&self) -> Box<dyn Any>;
}

impl dyn Val {
    pub fn downcast_ref<T: Val>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast<T: Val>(&self) -> Result<Box<T>, Box<dyn Any>> {
        self.as_boxed_any().downcast::<T>()
    }

    pub fn display(&self) -> String {
        format!("{}", self)
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

impl Clone for Box<dyn Val> {
    fn clone(&self) -> Self {
        let mut target: Box<dyn Val> = Box::new(false);
        Box::clone_into(&self, &mut target);

        target
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

    fn is_num(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
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

    fn is_num(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
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

    fn is_num(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
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

    fn is_str(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
    }
}

impl Val for bool {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_bool) = other.downcast_ref::<bool>() {
            *self && *other_bool
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

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct Tuple(Vec<Box<dyn Val>>);

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Val for Tuple {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(other_vec) = other.downcast_ref::<Tuple>() {
            self.0 == other_vec.0
        } else {
            false
        }
    }

    fn hash_val(&self, mut state: &mut dyn Hasher) {
        self.0.hash(&mut state);
    }

    fn is_tup(&self) -> bool {
        true
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_boxed_any(&self) -> Box<dyn Any> {
        Box::new(self.to_owned())
    }
}

pub trait Set: Val {
    fn is_finite(&self) -> bool;
    fn is_countable(&self) -> bool;
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

        // todo, add some extra stuff so that you can't just emulate it 
        // right now it hashes length, then values, then sorts
        // so yuo can emulate {{0, 1, 2}} as {3, 0, 1, 2}, and they should collide.
    }
}

impl fmt::Display for FiniteSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.elements)
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
}
