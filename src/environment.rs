use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::ops::BitOr;
use std::rc::Rc;

use crate::set::{canon, CanonSet, Set, SetPool};
use crate::value::Val;

/// What the symbol map stores.
/// 
/// If the type is initialized but not the value, then [`SymStore::Type`] is used, but once the value is declared, the type no longer matters, because the variable can't change, and thus [`SymStore::Value`] is enough (type is `{value}`).
#[derive(Debug, Clone)]
pub enum SymStore {
    Value(Box<dyn Val>),
    Type(Rc<CanonSet>),
    FuncType(Vec<Rc<CanonSet>>, Rc<CanonSet>)
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
#[derive(Clone)]
pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    symbols: HashMap<String, SymStore>
}

impl Env {
    pub fn new(parent: Option<Rc<RefCell<Self>>>) -> Self {
        Self {
            parent,
            symbols: HashMap::new()
        }
    }

    pub fn from_env(env: &Rc<RefCell<Self>>) -> Self {
        RefCell::borrow(env).clone()
    }

    pub fn get(&self, name: &str) -> Option<SymStore> {
        if self.symbols.get(name).is_some() {
            self.symbols.get(name).map(|name| name.to_owned())
        } else if let Some(env) = &self.parent {
            RefCell::borrow(env).get(name)
        } else {
            None
        }
    }

    pub fn get_set(&self, set_name: &str) -> Option<Rc<CanonSet>> {
        if let Some(SymStore::Value(set)) = self.symbols.get(set_name) {
            if let Some(actual) = set.downcast_ref::<Rc<CanonSet>>() {
                return Some(Rc::clone(actual))
            }
        }

        if let Some(env) = &self.parent {
            RefCell::borrow(env).get_set(set_name)
        } else {
            None
        }
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

    /// `set` must already be interned.
    pub fn insert_sym(&mut self, name: String, value: Box<dyn Val>) {
        let value = if let Some(set) = value.downcast_ref::<Rc<CanonSet>>() {
            Box::new(Rc::clone(set))
        } else {
            value
        };

        self.symbols.insert(name, SymStore::Value(value));
    }

    /// `set` must already be interned.
    pub fn insert_sym_type(&mut self, name: String, set: Rc<CanonSet>) {
        self.symbols.insert(
            name, 
            SymStore::Type(set)
        );
    }

    pub fn insert_sym_func_type(&mut self, name: String, args: Vec<Rc<CanonSet>>, codomain: Rc<CanonSet>) {
        self.symbols.insert(
            name,
            SymStore::FuncType(args, codomain)
        );
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Env")
            .field("parent", if let None = self.parent {
                &None::<()>
            } else {
                &Some("recursive [Env]")
            })
            .field("symbols", &self.symbols)
            .finish()
    }
}
