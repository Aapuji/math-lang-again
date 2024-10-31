use std::collections::HashMap;
use std::rc::Rc;

use crate::set::{canon, CanonSet, Set, SetPool};
use crate::value::{Arg, Val};

/// What the symbol map stores.
/// 
/// If the type is initialized but not the value, then [`SymStore::Type`] is used, but once the value is declared, the type no longer matters, because the variable can't change, and thus [`SymStore::Value`] is enough (type is `{value}`).
#[derive(Debug, Clone)]
pub enum SymStore {
    Value(Box<dyn Val>),
    Type(Rc<CanonSet>),
    FuncType(Vec<Arg>, Rc<CanonSet>)
}

impl SymStore {
    /// Returns if it is a subset of the given set.
    fn subset_of(&self, set: Rc<CanonSet>) -> bool {
        match self {
            Self::Value(value) => set.contains(value),
            Self::Type(typeset) => typeset.is_subset(&set),
            Self::FuncType(_, _) => false // todo
        }
    }
}

/// An environment of symbols
#[derive(Debug, Clone)]
pub struct Env<'t> {
    parent: Option<&'t Env<'t>>,
    symbols: HashMap<String, SymStore>
}

impl<'t> Env<'t> {
    pub fn new(parent: Option<&'t Self>) -> Self {
        Self {
            parent,
            symbols: HashMap::new()
        }
    }

    pub fn get(&self, name: &str) -> Option<&SymStore> {
        self.symbols.get(name)
    }

    pub fn get_set(&self, set_name: &str) -> Option<Rc<CanonSet>> {
        if let Some(SymStore::Value(set)) = self.symbols.get(set_name) {
            if let Some(actual) = set.downcast_ref::<Rc<CanonSet>>() {
                return Some(Rc::clone(actual))
            }
        }

        None
    }

    /// Returns if the given key is in the [`Env`].
    pub fn contains_key(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Returns if the symbol has a value assigned to it
    pub fn is_sym_assigned(&self, name: &str) -> bool {
        match self.symbols.get(name) {
            Some(SymStore::Value(_)) => true,
            _ => false
        }
    }

    pub fn insert_sym(&mut self, name: String, value: Box<dyn Val>, set_pool: &mut SetPool) {
        let value = if let Some(set) = value.downcast_ref::<Rc<CanonSet>>() {
            Box::new(set_pool.intern(canon(Rc::clone(set))))
        } else {
            value
        };

        self.symbols.insert(name, SymStore::Value(value));
    }

    pub fn insert_sym_type(&mut self, name: String, set: Rc<CanonSet>, set_pool: &mut SetPool) {
        self.symbols.insert(
            name, 
            SymStore::Type(set_pool.intern(set))
        );
    }

    pub fn insert_sym_func_type(&mut self, name: String, args: Vec<Arg>, codomain: Rc<CanonSet>) {
        self.symbols.insert(
            name,
            SymStore::FuncType(args, codomain)
        );
    }
}
