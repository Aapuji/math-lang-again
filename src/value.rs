use std::any::Any;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use num::{BigInt, BigRational, Complex};

use crate::set::Set;

pub trait Val: Any + Debug + Display + CloneBox {
    fn compare(&self, other: &dyn Val) -> bool;
    fn hash_val(&self, state: &mut dyn Hasher);

    fn is_num(&self) -> bool { false }
    fn is_str(&self) -> bool { false }
    fn is_tup(&self) -> bool { false }
    fn is_mat(&self) -> bool { false }
    fn is_set(&self) -> bool { false }

    fn as_any(&self) -> &dyn Any;
    fn as_boxed_any(&self) -> Box<dyn Any>;

    fn into_boxed_set(&self) -> Option<Box<dyn Set>> {
        None
    }
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

impl Eq for dyn Val {}

impl Hash for dyn Val {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_val(state);
    }
}

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn Val>;
}

impl<T> CloneBox for T
where 
    T: 'static + Val + Clone
{
    fn clone_box(&self) -> Box<dyn Val> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Val> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for dyn Val {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other)
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
pub struct Tuple(pub Vec<Box<dyn Val>>);

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        
        let mut i = self.0.len();
        for element in self.0.iter() {
            if i > 1 {
                write!(f, "{}, ", element)?;
            } else {
                write!(f, "{}", element)?;
            }

            i -= 1;
        }

        write!(f, "]")
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
