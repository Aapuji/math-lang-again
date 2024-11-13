use crate::{set::{FiniteSet, Set}, value::Val};

pub trait ValIterator {
    
}

impl Iterator for dyn ValIterator {
    type Item = Box<dyn Val>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct FiniteIterator {
    
}

impl From<FiniteSet> for FiniteIterator {
    fn from(value: FiniteSet) -> Self {
        todo!()
    }
}

pub struct InfiniteIterator {

}
