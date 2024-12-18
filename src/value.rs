use std::any::Any;
use std::cell::RefCell;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use num::{BigInt, BigRational, Complex};

use crate::ast::expr::{self, Expr};
use crate::environment::{Env, SymStore};
use crate::interpreter::Interpreter;
use crate::set::{CanonSet, Set};

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
}

impl dyn Val {
    pub fn downcast_ref<T: Val>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn downcast<T: Val>(&self) -> Result<Box<T>, Box<dyn Any>> {
        self.as_boxed_any().downcast::<T>()
    }

    pub fn display(&self) -> String {
        format!("{self}")
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

impl Display for Tuple {
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

#[derive(Debug, Clone)]
pub struct Func {
    env: Rc<RefCell<Env>>, // uses vec instead of hashmap because # of args is likely small and order by insertion is needed
    arg_names: Vec<String>,
    expr: Box<dyn Expr>,
    codomain: Rc<CanonSet>
}

impl Func {
    pub fn new(env: Rc<RefCell<Env>>, arg_names: &[String], expr: Box<dyn Expr>, interned_set: &Rc<CanonSet>) -> Self {
        Self {
            env,
            arg_names: arg_names.to_owned(),
            expr,
            codomain: Rc::clone(interned_set)
        }
    }

    pub fn from_func_expr(value: &expr::Func, parent: Rc<RefCell<Env>>) -> Self {
        let mut arg_names = Vec::with_capacity(value.0.len());
        let mut env = Env::new(Some(Rc::clone(&parent)));
        
        for sym in &value.0 {
            env.insert_sym_type(sym.0.to_owned(), parent.borrow().get_set("Univ").unwrap());
            arg_names.push(sym.0.to_owned());
        }

        Self {
            env: Rc::new(RefCell::new(env)),
            arg_names,
            expr: value.1.to_owned(),
            codomain: parent.borrow().get_set("Univ").unwrap()
        }
    }

    pub fn clone_with_env(&self, new_env: Rc<RefCell<Env>>, ) -> Self {
        Self {
            env: new_env,
            arg_names: self.arg_names.clone(),
            expr: self.expr.clone(),
            codomain: self.codomain.clone()
        }
    }

    pub fn call(&self, args: &[Option<Box<dyn Val>>]) -> Box<dyn Val> {
        if args.len() > self.arity() {
            panic!("Too many arguments")
        }

        let mut call_env = self.env.borrow().clone();

        let mut curried_args = vec![];

        for (i, arg) in args.iter().enumerate() {
            if let Some(val) = arg {
                let arg_name = &self.arg_names[i];
                
                if let Some(SymStore::Type(typeset)) = self.env.borrow().get(arg_name) {
                    if !typeset.contains(val) {
                        panic!("Parameter '{arg_name}' belongs to '{typeset}' which doesn't contain '{val}'");
                    }
                } else {
                    unreachable!()
                }

                call_env.insert_sym(arg_name.clone(), val.to_owned());
            } else {
                curried_args.push(self.arg_names[i].clone());
            }
        }

        if args.len() < self.arity() {
            for i in args.len()..self.arity() {
                curried_args.push(self.arg_names[i].clone());
            }
        }

        let call_env = Rc::new(RefCell::new(call_env));

        let mut interpreter = Interpreter::with_env(&call_env);

        if curried_args.len() > 0 {
            return Box::new(
                Self {
                    env: Rc::clone(&call_env),
                    expr: interpreter.curry_expr(&self.expr, &curried_args.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
                    arg_names: curried_args,
                    codomain: Rc::clone(&self.codomain)
                }
            )
        }

        interpreter.execute_expr(&self.expr)
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.env.borrow().contains_key(name)
    }

    pub fn define(&mut self, name: &str, value: Box<dyn Val>) {
        self.env.borrow_mut().insert_sym(name.to_owned(), value);
    }

    pub fn arity(&self) -> usize {
        self.arg_names.len()
    }

    pub fn env(&self) -> &Rc<RefCell<Env>> {
        &self.env
    }

    pub fn args(&self) -> &[String] {
        &self.arg_names
    }

    pub fn expr(&self) -> &Box<dyn Expr> {
        &self.expr
    }

    pub fn codomain(&self) -> &Rc<CanonSet> {
        &self.codomain
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.arity() == 1 {
            write!(f, "{} -> ", self.args()[0])
        } else {
            let mut s = String::from("(");

            for (i, a) in self.args().iter().enumerate() {
                if i == self.arity() - 1 {
                    s.push_str(&format!("{}", a));
                } else {
                    s.push_str(&format!("{}, ", a));
                }
            }

            s.push(')');

            write!(f, "{s} -> ")
        }?;

        write!(f, "{}", self.expr)
    }
}

impl Hash for Func {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl Val for Func {
    fn compare(&self, other: &dyn Val) -> bool {
        if let Some(func @ Func { .. }) = other.downcast_ref() {
            true // todo
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
