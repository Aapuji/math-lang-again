use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Neg;
use std::rc::Rc;
use num::bigint::Sign;
use num::{BigInt, BigRational, Complex, One, Zero};
use num::pow::Pow;

use crate::ast::{expr, expr::*, stmt::*};
use crate::environment::{Env, SymStore};
use crate::set::{self, canon, CanonSet, FiniteSet, InfiniteSet, Set, SetPool};
use crate::token::{Token, TokenKind};
use crate::types;
use crate::value::{Func, Tuple, Val};

#[derive(Debug)]
pub struct Interpreter {
    env: Rc<RefCell<Env>>,
    set_pool: SetPool
}

macro_rules! insert_set {
    (
        $env:ident ;
        $name:ident :
        $set:expr ;
        $set_pool:ident
    ) => {
        $env.insert_sym(
            String::from(stringify!($name)),
            Box::new($set_pool.intern(&Rc::new($set)))
        )
    };
}

impl Interpreter {
    pub fn new() -> Self {
        let mut set_pool = SetPool::new();
        let mut env = Env::new(None);

        // All-encompassing Types
        insert_set!(env; Univ: CanonSet::Infinite(InfiniteSet::Univ); set_pool);
        insert_set!(env; Empty: CanonSet::Finite(FiniteSet::new(HashSet::new())); set_pool);

        // Numeric Types (implementing class Num?)
        insert_set!(env; Nat: CanonSet::Infinite(InfiniteSet::Nat); set_pool);
        insert_set!(env; Int: CanonSet::Infinite(InfiniteSet::Int); set_pool);
        insert_set!(env; Real: CanonSet::Infinite(InfiniteSet::Real); set_pool);
        insert_set!(env; Complex: CanonSet::Infinite(InfiniteSet::Complex); set_pool);

        // Text Types (implementing class Text?)
        insert_set!(env; Str: CanonSet::Infinite(InfiniteSet::Str); set_pool);

        Self {
            env: Rc::new(RefCell::new(env)),
            set_pool
        }
    }

    pub fn with_env(env: &Rc<RefCell<Env>>) -> Self {
        Self {
            env: Rc::clone(env),
            set_pool: SetPool::new()
        }
    }

    pub fn interpret<'s>(&mut self, stmts: &'s [Box<dyn Stmt>]) {
        for stmt in stmts {
            self.execute_stmt(stmt);
        }
    }

    pub fn execute_stmt(&mut self, stmt: &Box<dyn Stmt>) {
        if let Some(ExprStmt(expr, is_to_log)) = stmt.downcast_ref() {
            // assign
            if let Some(Assign(Symbol(name), right)) = expr.downcast_ref() {
                let value = self.execute_assign(name, right);

                if *is_to_log {
                    println!("{name} = {value}")
                }
            // typed assign
            } else if let Some(TypedAssign(Symbol(name), typeset, right)) = expr.downcast_ref() {
                self.execute_typed_assign(name, typeset, right);
            // type expr : typecast or typedef
            } else if let Some(TypeExpr(value, typeset)) = expr.downcast_ref() {
                if let Some(Symbol(name)) = value.downcast_ref() {
                    if !RefCell::borrow(&self.env).is_sym_assigned(name) {
                        let typeset = self.execute_expr(typeset);

                        // type def
                        if let Some(set) = typeset.downcast_ref::<Rc<CanonSet>>() {
                            self.env.borrow_mut().insert_sym_type(name.to_owned(), Rc::clone(&self.set_pool.intern(set)));
                            return;
                        } else {
                            panic!("'{typeset}' is not a set")
                        }
                    }
                }

                // type cast
                println!("{}", todo!());
            
            } else if let Some(FuncTypeExpr(func, arg_types, codom)) = expr.downcast_ref() {
                if let Some(Symbol(name)) = func.downcast_ref() {
                    /* perhaps there will have to be a check for only defined in the current env
                    because the following could happen:

                    f : Real -> Real
                    f(x) = x + 1

                    do 
                        f : Real, Real -> Real
                        f(x, y) = x + y
                    end f(0, 1)

                    Or do we not allow that either??
                    */

                    if !RefCell::borrow(&self.env).is_sym_assigned(name) {
                        let mut dom_arr = Vec::with_capacity(arg_types.len());

                        for typeset in arg_types {
                            let typeset = self.execute_expr(typeset);

                            if let Some(set) = typeset.downcast_ref::<Rc<CanonSet>>() {
                                dom_arr.push(set.to_owned());
                            } else {
                                panic!("'{typeset}' is not a set")
                            }
                        }

                        let codom = self.execute_expr(codom);
                        if let Some(set) = codom.downcast_ref::<Rc<CanonSet>>() {
                            self.env.borrow_mut().insert_sym_func_type(name.to_owned(), dom_arr, Rc::clone(set));
                            return;
                        } else {
                            panic!("'{codom}' is not a set")
                        }
                    }
                }

                // function type cast???
                println!("{}", todo!());

            } else {
                println!("{}", self.execute_expr(expr));
            }
        } else {
            todo!()
        }
    }

    pub fn execute_expr<'a>(&mut self, expr: &'a Box<dyn Expr>) -> Box<dyn Val> {
        if let Some(Literal(lit)) = expr.downcast_ref() {
            Self::execute_literal(lit)
        } else if let Some(Symbol(name)) = expr.downcast_ref() {
            if let Some(SymStore::Value(value)) = RefCell::borrow(&self.env).get(name) {
                value.clone()
            } else {
                panic!("Variable '{name}' is not defined");
            }
        } else if let Some(Group(expr)) = expr.downcast_ref::<Group>() {
            self.execute_expr(expr)
        } else if let Some(Unary(op, right)) = expr.downcast_ref() {
            let right = self.execute_expr(right);

            if let Some(func) = right.downcast_ref::<Func>() {
                return Box::new(Func::new(
                    Rc::clone(func.env()), 
                    func.args(), 
                    Box::new(Unary(op.clone(), Box::new(Group(func.expr().to_owned())))),
                    func.codomain()
                ));
            }

            match op.kind() {
                &TokenKind::Minus => Self::execute_neg(&right),
                _ => todo!()
            }
        } else if let Some(Binary(left, op, right)) = expr.downcast_ref() {
            let left = self.execute_expr(left);
            let right = self.execute_expr(right);

            if let Some(l_func) = left.downcast_ref::<Func>() {
                // right is a function
                if let Some(r_func) = right.downcast_ref::<Func>() {
                    if l_func.arity() == r_func.arity() {
                        let mut new_expr = r_func.expr().to_owned();
                        Self::substitute_symbols(
                            &mut new_expr, 
                            &r_func.args().iter().map(|s| s.as_str()).collect::<Vec<_>>()[..], 
                            l_func.args()
                        );

                        return Box::new(Func::new(
                            Rc::clone(l_func.env()),
                            l_func.args(),
                            Box::new(Binary(
                                Box::new(Group(l_func.expr().to_owned())),
                                op.to_owned(),
                                Box::new(Group(new_expr))
                            )),
                            &RefCell::borrow(&self.env).get_set("Univ").unwrap() // later do some math stuff here i guess
                        ))
                    } else {
                        panic!("Function shorthand can only be used with functions with the same arity.")
                    }
                }

                // right is not a function
                return Box::new(Func::new(
                    Rc::clone(l_func.env()),
                    l_func.args(),
                    Box::new(Binary(
                        Box::new(Group(l_func.expr().to_owned())),
                        op.to_owned(),
                        Box::new(Literal(right))
                    )),
                    &RefCell::borrow(&self.env).get_set("Univ").unwrap() // later do some math stuff here i guess
                ))
            } else if let Some(r_func) = right.downcast_ref::<Func>() {
                // left is not a function
                return Box::new(Func::new(
                    Rc::clone(r_func.env()),
                    r_func.args(),
                    Box::new(Binary(
                        Box::new(Literal(left)),
                        op.to_owned(),
                        Box::new(Group(r_func.expr().to_owned()))
                    )),
                    &RefCell::borrow(&self.env).get_set("Univ").unwrap() // later do some math stuff here i guess
                ))
            }

            match op.kind() {
                &TokenKind::Plus    => Self::execute_sum(&left, &right),
                &TokenKind::Minus   => Self::execute_diff(&left, &right),
                &TokenKind::Star    => Self::execute_prod(&left, &right),
                &TokenKind::Slash   => Self::execute_quot(&left, &right),
                &TokenKind::Caret   => Self::execute_power(&left, &right),
                _ => todo!()
            }
        } else if let Some(expr::Tuple(exprs)) = expr.downcast_ref() {
            Box::new(Tuple(exprs
                .iter()
                .map(|expr| self.execute_expr(expr))
                .collect::<Vec<Box<dyn Val>>>()))
        } else if let Some(expr::Set(values)) = expr.downcast_ref() {
            self.execute_set(values)
        } else if let Some(func) = expr.downcast_ref::<expr::Func>() {
            Box::new(Func::from_func_expr(func, Rc::clone(&self.env)))
        } else if let Some(Call(func_expr, arg_exprs)) = expr.downcast_ref() {
            let func_value = self.execute_expr(func_expr);

            if let Some(func) = func_value.downcast_ref::<Func>() {
                let args = arg_exprs
                    .iter()
                    .map(|arg| if let Some(actual) = arg {
                        Some(self.execute_expr(actual))
                    } else {
                        None
                    })
                    .collect::<Vec<_>>();

                func.call(&args)
            } else {
                panic!("'{func_value}' is not callable")
            }
        } else {
            todo!()
        }
    }

    /// Is similar to [`Interpreter::execute_expr`], but doesn't actually execute any expression, but instead just replaces all symbols that aren't in the given `symbols` slice with their actual values.
    pub fn curry_expr<'a>(&mut self, expr: &'a Box<dyn Expr>, symbols: &[&str]) -> Box<dyn Expr> {
        if let Some(Literal(lit)) = expr.downcast_ref() {
            expr.to_owned()
        } else if let Some(Symbol(name)) = expr.downcast_ref() {
            if let Some(SymStore::Value(value)) = RefCell::borrow(&self.env).get(name) {
                if !symbols.contains(&name.as_str()) {
                    Box::new(Literal(value.clone()))
                } else {
                    expr.clone()
                }
            } else if let Some(SymStore::Type(_)) = RefCell::borrow(&self.env).get(name) {
                expr.clone()
            } else {
                panic!("Variable '{name}' is not defined")
            }
        } else if let Some(Group(expr)) = expr.downcast_ref::<Group>() {
            Box::new(Group(self.curry_expr(expr, symbols)))
        } else if let Some(Unary(op, right)) = expr.downcast_ref() {
            Box::new(Unary(op.to_owned(), self.curry_expr(right, symbols)))
        } else if let Some(Binary(left, op, right)) = expr.downcast_ref() {
            Box::new(Binary(self.curry_expr(left, symbols), op.to_owned(), self.curry_expr(right, symbols)))
        } else if let Some(expr::Tuple(exprs)) = expr.downcast_ref() {
            Box::new(expr::Tuple(exprs
                .iter()
                .map(|expr| self.curry_expr(expr, symbols))
                .collect::<Vec<Box<dyn Expr>>>()))
        } else if let Some(expr::Set(values)) = expr.downcast_ref() {
            Box::new(expr::Set(values.iter().map(|x| self.curry_expr(x, symbols)).collect()))
        } else if let Some(expr::Func(args, result)) = expr.downcast_ref::<expr::Func>() {
            todo!() // this may be a bit more compelx

            // Box::new(Func::from_func_expr(func, Rc::clone(&self.env), &mut self.set_pool))
        } else if let Some(Call(func_expr, arg_exprs)) = expr.downcast_ref() {
            let curry_func_expr = self.curry_expr(func_expr, symbols);
            let curry_args = arg_exprs
                .iter()
                .map(|a| if let Some(actual) = a {
                    Some(self.curry_expr(actual, symbols))
                } else {
                    None
                })
                .collect();

            Box::new(Call(curry_func_expr, curry_args))
        } else {
            todo!()
        }
    }

    /// Substitutes all instances of symbols in `find_args` with their corresponding symbol in `replace_with`.
    /// 
    /// Thus, `find_args.len() == replace_with.len()`.
    fn substitute_symbols(expr: &mut Box<dyn Expr>, find_args: &[&str], replace_with: &[String]) {
        if let Some(_) = expr.downcast_ref::<Literal>() {
            ()
        } else if let Some(symbol) = expr.downcast_mut::<Symbol>() {
            if let Some(i) = find_args.iter().position(|name| name == &symbol.0) {
                symbol.0 = replace_with[i].clone()
            }
        } else if let Some(Group(inner)) = expr.downcast_mut() {
            Self::substitute_symbols(inner, find_args, replace_with);
        } else if let Some(Unary(_, operand)) = expr.downcast_mut() {
            Self::substitute_symbols(operand, find_args, replace_with);
        } else if let Some(Binary(left, _, right)) = expr.downcast_mut() {
            Self::substitute_symbols(left, find_args, replace_with);
            Self::substitute_symbols(right, find_args, replace_with);
        } else if let Some(Call(func, args)) = expr.downcast_mut() {
            Self::substitute_symbols(func, find_args, replace_with);

            for arg in args.iter_mut() {
                if let Some(actual) = arg {
                    Self::substitute_symbols(actual, find_args, replace_with);
                }
            }
        } else if let Some(expr::Func(args, inner)) = expr.downcast_mut() {
            let mut new_find_args: Vec<&str> = Vec::with_capacity(find_args.len());
            let mut new_replace_with = Vec::with_capacity(replace_with.len());
            
            for (i, arg) in find_args.iter().enumerate() {
                if let Some(_) = args.iter().position(|a| a.0 == *arg) {
                    ()
                } else {
                    new_find_args.push(arg);
                    new_replace_with.push(replace_with[i].clone())
                }
            }

            Self::substitute_symbols(inner, &new_find_args[..], &new_replace_with);
        } else if let Some(expr::Tuple(exprs)) = expr.downcast_mut() {
            for x in exprs {
                Self::substitute_symbols(x, find_args, replace_with);
            }
        } else if let Some(Matrix(mat)) = expr.downcast_mut() {
            for row in mat {
                for x in row {
                    Self::substitute_symbols(x, find_args, replace_with);
                }
            }
        } else if let Some(_) = expr.downcast_mut::<expr::Set>() {
            todo!() // may get a bit weird?
        } else {
            todo!()
        }
    }

    fn execute_literal(lit: &Box<dyn Val>) -> Box<dyn Val> {
        if let Ok(bigint) = lit.downcast::<BigInt>() {
            bigint
        } else if let Ok(bigrat) = lit.downcast::<BigRational>() {
            bigrat
        } else if let Ok(complex) = lit.downcast::<Complex<BigRational>>() {
            complex
        } else if let Ok(string) = lit.downcast::<String>(){
            string
        } else if let Ok(bool) = lit.downcast::<bool>() {
            bool
        } else {
            todo!()
        }
    }

    fn execute_neg(right: &Box<dyn Val>) -> Box<dyn Val> {
        // Numbers -x
        if let Some(bigint) = right.downcast_ref::<BigInt>() {
            Box::new(-bigint)
        } else if let Some(bigrat) = right.downcast_ref::<BigRational>() {
            Box::new(-bigrat)
        } else if let Some(complex) = right.downcast_ref::<Complex<BigRational>>() {
            Box::new(-complex)
        } else if let Some(&bool) = right.downcast_ref::<bool>() {
            Box::new(bool)
        } else {
            panic!("Cannot apply unary operator '-'");
        }
    }

    fn execute_sum(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {
        // String + _
        if let Ok(l_str) = left.downcast::<String>() {
            Box::new(*l_str + &right.display())
        // _ + String
        } else if let Ok(r_str) = right.downcast::<String>() {
            Box::new(left.display() + &*r_str)
        // BigInt + _
        } else if let Ok(l_bigint) = left.downcast::<BigInt>() {
            // Adding BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bigint + *r_bigint)
            // Adding BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(BigRational::from(*l_bigint) + *r_bigrat)
            // Adding Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(BigRational::from(*l_bigint)) + *r_complex)
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bigint + *r_bool as u8)
            } else {
                panic!("Cannot apply binary operator '+'")
            }
        // BigRational + _
        } else if let Ok(l_bigrat) = left.downcast::<BigRational>() {
            // Adding BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bigrat + *r_bigint)
            // Adding BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_bigrat + *r_bigrat)
            // Adding Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(*l_bigrat) + *r_complex)
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bigrat + BigInt::from(*r_bool as u8))
            } else {
                panic!("Cannot apply binary operator '+'")
            }
        // Complex + _
        } else if let Ok(l_complex) = left.downcast::<Complex<BigRational>>() {
            // Adding BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_complex + Complex::<BigRational>::from(BigRational::from(*r_bigint)))
            // Adding BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_complex + *r_bigrat)
            // Adding Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(*l_complex + *r_complex)
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_complex + BigRational::from(BigInt::from(*r_bool as u8)))
            } else {
                panic!("Cannot apply binary operator '+'")
            }
        } else if let Ok(l_bool) = left.downcast::<bool>() {
            // Adding BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bool as u8 + *r_bigint)
            // Adding BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(BigRational::from(BigInt::from(*l_bool as u8)) + *r_bigrat)
            // Adding Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(BigRational::from(BigInt::from(*l_bool as u8))) + *r_complex)
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bool ^ *r_bool)
            } else {
                panic!("Cannot apply binary operator '+'")
            }
        } else {
            panic!("Cannot apply binary operator '+'")
        }
    }

    fn execute_diff(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {
        if let Ok(_) = left.downcast::<String>() {
            panic!("Cannot subtract from a string")
        } else if let Ok(_) = right.downcast::<String>() {
            panic!("Cannot subtract a string")
        } else if left.is_num() && right.is_num() {
            let right = Self::execute_neg(right);

            Self::execute_sum(left, &right)
        } else {
            todo!()
        }
    }

    fn execute_prod(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {
        if let Ok(_) = left.downcast::<String>() {
            // Will have to check if the value belongs to Nat and then repeat the string if so
            todo!()
        } else if right.is_str() {
            panic!("Cannot multiply by a string")
        } else if let Ok(l_bigint) = left.downcast::<BigInt>() {
            // Adding BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bigint * *r_bigint)
            // Adding BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(BigRational::from(*l_bigint) * *r_bigrat)
            // Adding Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(BigRational::from(*l_bigint)) * *r_complex)
            // Adding Bool
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bigint * *r_bool as u8)
            } else {
                panic!("Cannot apply binary operator '*'")
            }
        // BigRational + _
        } else if let Ok(l_bigrat) = left.downcast::<BigRational>() {
            // Multiplying BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bigrat * *r_bigint)
            // Multiplying BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_bigrat * *r_bigrat)
            // Multiplying Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(*l_bigrat) * *r_complex)
            // Multiplying Bool
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bigrat * BigInt::from(*r_bool as u8))
            } else {
                panic!("Cannot apply binary operator '*'")
            }
        // Complex * _
        } else if let Ok(l_complex) = left.downcast::<Complex<BigRational>>() {
            // Multiplying BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_complex * Complex::<BigRational>::from(BigRational::from(*r_bigint)))
            // Multiplying BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_complex * *r_bigrat)
            // Multiplying Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(*l_complex * *r_complex)
            // Multiplying Bool
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_complex * BigRational::from(BigInt::from(*r_bool as u8)))
            } else {
                panic!("Cannot apply binary operator '*'")
            }
        } else if let Ok(l_bool) = left.downcast::<bool>() {
            // Multiplying BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bool as u8 * *r_bigint)
            // Multiplying BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(BigRational::from(BigInt::from(*l_bool as u8)) * *r_bigrat)
            // Multiplying Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(BigRational::from(BigInt::from(*l_bool as u8))) * *r_complex)
            } else if let Ok(r_bool) = right.downcast::<bool>() {
                Box::new(*l_bool & *r_bool)
            } else {
                panic!("Cannot apply binary operator '*'")
            }
        } else {
            panic!("Cannot apply binary operator '*'")
        }
    }

    fn execute_quot(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {
        if left.is_str() || right.is_str() {
            panic!("Cannot apply binary operator '/' to text")
        // BigInt / _
        } else if let Ok(l_bigint) = left.downcast::<BigInt>() {
            // Dividing BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(BigRational::new(*l_bigint, *r_bigint))
            // Dividing BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(BigRational::from(*l_bigint) / *r_bigrat)
            // Dividing Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(BigRational::from(*l_bigint)) / *r_complex)
            // Cannot Divide by Bools
            } else if let Ok(_) = right.downcast::<bool>() {
                panic!("Cannot divide by a boolean")
            } else {
                panic!("Cannot apply binary operator '/'")
            }
        // BigRational / _
        } else if let Ok(l_bigrat) = left.downcast::<BigRational>() {
            // Dividing BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_bigrat / *r_bigint)
            // Dividing BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_bigrat / *r_bigrat)
            // Dividing Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(Complex::<BigRational>::from(*l_bigrat) / *r_complex)
            // Cannot Divide by Bools
            } else if let Ok(_) = right.downcast::<bool>() {
                panic!("Cannot divide by a boolean")
            } else {
                panic!("Cannot apply binary operator '/'")
            }
        // Complex / _
        } else if let Ok(l_complex) = left.downcast::<Complex<BigRational>>() {
            // Dividing BigInt
            if let Ok(r_bigint) = right.downcast::<BigInt>() {
                Box::new(*l_complex / Complex::<BigRational>::from(BigRational::from(*r_bigint)))
            // Dividing BigRational
            } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                Box::new(*l_complex / *r_bigrat)
            // Dividing Complex
            } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                Box::new(*l_complex / *r_complex)
            // Cannot Divide by Bools
            } else if let Ok(_) = right.downcast::<bool>() {
                panic!("Cannot divide by a boolean")
            } else {
                panic!("Cannot apply binary operator '/'")
            }
        // Cannot use division with booleans
        } else if let Ok(_) = left.downcast::<bool>() {
            panic!("Cannot use division with booleans")
        } else {
            panic!("Cannot apply binary operator '/'")
        }
    }

    fn execute_power(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {            
        if let Some(set) = left.downcast_ref::<Rc<CanonSet>>() {
            if InfiniteSet::Nat.contains(right) {
                todo!()
            } else {
                panic!("'{right}' is not in 'Nat'");
            }
        } else {
            if left.is_str() || right.is_str() {
                panic!("Cannot apply binary operator '^' to text")
            // BigInt ^ _
            } else if let Ok(l_bigint) = left.downcast::<BigInt>() {
                // Exponentiating BigInt
                if let Ok(r_bigint) = right.downcast::<BigInt>() {
                    if *r_bigint == BigInt::zero() {
                        if *l_bigint == BigInt::zero() {
                            panic!("Cannot raise '0' to the power of '0'")
                        } else {
                            return Box::new(BigInt::one())
                        }
                    }

                    let v = r_bigint.to_u32_digits();

                    // Don't have to check if v.1.len() > 1 for r_x = 0 or 1, because the len for them won't be > 1

                    let res: Box<dyn Val>;
                    if v.0 != Sign::Minus {
                        res = Box::new(l_bigint.pow(if v.1.len() > 1 {
                            panic!("Exponent is too large to compute");
                        } else {
                            v.1[0]
                        }))
                    } else {
                        if *l_bigint == BigInt::zero() {
                            panic!("Base of negative exponent cannot be '0'")
                        } else if *l_bigint == BigInt::one() {
                            res = Box::new(BigInt::one());
                        } else if v.1.len() > 1 {
                            // approximate with pow=-inf, aka result=0
                            res = Box::new(BigInt::zero())
                        } else {
                            res = Box::new(BigRational::new(BigInt::one(), l_bigint.pow(v.1[0])))
                        }
                    };

                    res
                // Exponentiating BigRational
                } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                    todo!()
                // Exponentiating Complex
                } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                    todo!()
                // Exponentiating Bools
                } else if let Ok(bool) = right.downcast::<bool>() {
                    Box::new(l_bigint.pow(*bool as u32))
                } else {
                    panic!("Cannot apply binary operator '^'")
                }
            // BigRational ^ _
            } else if let Ok(l_bigrat) = left.downcast::<BigRational>() {
                // Exponentiating BigInt
                if let Ok(r_bigint) = right.downcast::<BigInt>() {
                    if *r_bigint == BigInt::zero() {
                        if *l_bigrat == BigRational::zero() {
                            panic!("Cannot raise '0' to the power of '0'")
                        } else {
                            return Box::new(BigInt::one())
                        }
                    }
                    
                    let v = r_bigint.to_u32_digits();
                    let res: Box<dyn Val>;

                    // left > 1
                    if *l_bigrat >= BigRational::one() {
                        if v.0 != Sign::Minus {
                            if v.1.len() > 1 {
                                panic!("Exponent is too large to compute")
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]))
                            }
                        } else {
                            if v.1.len() > 1 {
                                // approximate with result=0
                                res = Box::new(BigInt::zero())
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]).recip())
                            }
                        }
                    // 0 < left < 1
                    } else if *l_bigrat > BigRational::zero() {
                        if v.0 != Sign::Minus {
                            if v.1.len() > 1 {
                                // approximate with result=0
                                res = Box::new(BigInt::zero())
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]))
                            }
                        } else {
                            if v.1.len() > 1 {
                                panic!("Exponent is too large to compute")
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]).recip())
                            }
                        }
                    // left == 0
                    } else if *l_bigrat == BigRational::zero() {
                        if v.0 != Sign::Minus {
                            if v.1.len() > 1 {
                                res = Box::new(BigInt::zero())
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]))
                            }
                        } else {
                            panic!("Base of negative exponent cannot be '0'")
                        }
                    // -1 < left < 0
                    } else if *l_bigrat > BigRational::one().neg() {
                        if v.0 != Sign::Minus {
                            if v.1.len() > 1 {
                                // approx with result=0
                                res = Box::new(BigInt::zero())
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]))
                            }
                        } else {
                            if v.1.len() > 1 {
                                panic!("Exponent too large to compute")
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]).recip())
                            }
                        }
                    // left == -1 : flips between 1 and -1
                    } else if *l_bigrat == BigRational::one().neg() {
                        if *r_bigint % 2 == BigInt::zero() {
                            res = Box::new(BigInt::one())
                        } else {
                            res = Box::new(BigInt::one().neg())
                        }
                    // left < -1
                    } else {
                        if v.0 != Sign::Minus {
                            if v.1.len() > 1 {
                                panic!("Exponent too large to compute")
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]))
                            }
                        } else {
                            if v.1.len() > 1 {
                                // approx with result=0
                                res = Box::new(BigInt::zero())
                            } else {
                                res = Box::new(l_bigrat.pow(v.1[0]).recip())
                            }
                        }
                    }

                    res
                // Exponentiating BigRational
                } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                    todo!()
                // Exponentiating Complex
                } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                    todo!()
                // Exponentiating Bools
                } else if let Ok(bool) = right.downcast::<bool>() {
                    Box::new(l_bigrat.pow(*bool as u32))
                } else {
                    panic!("Cannot apply binary operator '^'")
                }
            // Complex ^ _
            } else if let Ok(l_complex) = left.downcast::<Complex<BigRational>>() {
                // Dividing BigInt
                if let Ok(r_bigint) = right.downcast::<BigInt>() {
                    todo!()
                // Dividing BigRational
                } else if let Ok(r_bigrat) = right.downcast::<BigRational>() {
                    todo!()
                // Exponentiating Complex
                } else if let Ok(r_complex) = right.downcast::<Complex<BigRational>>() {
                    todo!()
                // Exponentiating Bools
                } else if let Ok(_) = right.downcast::<bool>() {
                    todo!()
                } else {
                    panic!("Cannot apply binary operator '^'")
                }
            // Cannot use division with booleans
            } else if let Ok(_) = left.downcast::<bool>() {
                panic!("Cannot use division with booleans")
            } else {
                panic!("Cannot apply binary operator '/'")
            }
        }
    }

    fn execute_set(&mut self, exprs: &[Box<dyn Expr>]) -> Box<dyn Val> {
        let mut set = HashSet::<Box<dyn Val>>::new();

        for expr in exprs {
            set.insert(self.execute_expr(expr));
        }

        Box::new(Rc::new(CanonSet::Finite(FiniteSet::new(set))))
    }

    fn execute_assign(&mut self, name: &str, right: &Box<dyn Expr>) -> Box<dyn Val> {
        if RefCell::borrow(&self.env).is_sym_assigned(name) {
            panic!("Variable {name} cannot be reassigned")
        }

        let mut right = self.execute_expr(right);

        if let Ok(func) = right.downcast::<Func>() {
            // function name already has a map type
            if let Some(SymStore::FuncType(arg_types, codomain)) = self.env.borrow_mut().get(name) {
                if func.arity() != arg_types.len() {
                    panic!("Function '{name}' was previously denoted to have {} arguments, but is declared to have {} instead.", arg_types.len(), func.arity())
                }

                let mut new_env = Env::from_env(func.env());

                for (i, typeset) in arg_types.iter().enumerate() {
                    let arg_name = &func.args()[i];
                    
                    new_env.insert_sym_type(arg_name.to_owned(), self.set_pool.intern(typeset));
                }

                right = Box::new(func.clone_with_env(Rc::new(RefCell::new(new_env))));
            }
        } else {
            
            if let Some(SymStore::Type(typeset)) = RefCell::borrow(&self.env).get(name) {
                if !typeset.contains(&right) {
                    panic!("'{name}' is in '{typeset}' which does not contain '{right}'")
                }
            }
        }

        self.env.borrow_mut().insert_sym(
            name.to_owned(),
            right.clone()
        );

        right
    }

    fn execute_typed_assign(&mut self, name: &str, typeset: &Box<dyn Expr>, right: &Box<dyn Expr>) {
        if RefCell::borrow(&self.env).is_sym_assigned(name) {
            panic!("Variable '{name}' cannot be reassigned")
        }

        let typeset = self.execute_expr(typeset);

        if let Some(set) = typeset.downcast_ref::<Rc<CanonSet>>() {
            let value = self.execute_expr(right);

            if set.contains(&value) {
                self.env.borrow_mut().insert_sym(name.to_owned(), value);
            } else {
                panic!("Incompatible types: '{value}' cannot be cast into '{typeset}'");
            }
        } else {
            panic!("'{typeset}' is not a set");
        }
    }
}
