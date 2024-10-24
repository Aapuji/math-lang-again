use std::collections::{HashMap, HashSet};
use std::ops::Neg;
use std::rc::Rc;
use num::bigint::Sign;
use num::{BigInt, BigRational, Complex, One, Zero};
use num::pow::Pow;

use crate::ast::{expr, expr::*, stmt::*};
use crate::set::{self, canon, CanonSet, FiniteSet, InfiniteSet, Set, SetPool};
use crate::token::TokenKind;
use crate::value::{Func, FuncArg, Tuple, Val};

/// What the symbol map stores.
/// 
/// If the type is initialized but not the value, then [`SymStore::Type`] is used, but once the value is declared, the type no longer matters, because the variable can't change, and thus [`SymStore::Value`] is enough (type is `{value}`).
#[derive(Debug)]
enum SymStore {
    Value(Box<dyn Val>),
    Type(Rc<CanonSet>),
    FuncType(Vec<FuncArg>, Rc<CanonSet>)
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

#[derive(Debug)]
pub struct Interpreter {
    symbols: HashMap<String, SymStore>,
    set_pool: SetPool
}

impl Interpreter {
    pub fn new() -> Self {
        let mut this = Self { 
            symbols: HashMap::new(),
            set_pool: SetPool::new()
        };

        // All-encompassing Types
        this.insert_sym(String::from("Univ"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Univ))));
        this.insert_sym(String::from("Empty"), Box::new(Rc::new(CanonSet::Finite(FiniteSet::new(HashSet::new())))));

        // Numeric Types (implementing class Num?)
        this.insert_sym(String::from("Nat"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Nat))));
        this.insert_sym(String::from("Int"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Int))));
        this.insert_sym(String::from("Real"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Real))));
        this.insert_sym(String::from("Complex"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Complex))));

        // Text Types (implementing class Text?)
        this.insert_sym(String::from("Str"), Box::new(Rc::new(CanonSet::Infinite(InfiniteSet::Str))));

        this
    }

    fn is_sym_assigned(&self, name: &str) -> bool {
        match self.symbols.get(name) {
            Some(SymStore::Value(_)) => true,
            _ => false
        }
    }

    fn insert_sym(&mut self, name: String, value: Box<dyn Val>) {
        let value = if let Some(set) = value.downcast_ref::<Rc<CanonSet>>() {
            Box::new(self.set_pool.intern(canon(Rc::clone(set))))
        } else {
            value
        };

        self.symbols.insert(name, SymStore::Value(value));
    }

    fn insert_sym_type(&mut self, name: String, set: Rc<CanonSet>) {
        self.symbols.insert(
            name, 
            SymStore::Type(self.set_pool.intern(set))
        );
    }

    fn insert_sym_func_type(&mut self, name: String, args: Vec<FuncArg>, codomain: Rc<CanonSet>) {
        self.symbols.insert(
            name,
            SymStore::FuncType(args, codomain)
        );
    }

    pub fn interpret<'t>(&mut self, stmts: &'t [Box<dyn Stmt>]) {
        for stmt in stmts {
            self.execute_stmt(stmt);
        }
    }

    pub fn execute_stmt(&mut self, stmt: &Box<dyn Stmt>) {
        if let Some(ExprStmt(expr)) = stmt.downcast_ref() {
            // assign
            if let Some(Assign(Symbol(name), right)) = expr.downcast_ref() {
                self.execute_assign(name, right);
            // typed assign
            } else if let Some(TypedAssign(Symbol(name), typeset, right)) = expr.downcast_ref() {
                self.execute_typed_assign(name, typeset, right);
            // type expr : typecast or typedef
            } else if let Some(TypeExpr(value, typeset)) = expr.downcast_ref() {
                if let Some(Symbol(name)) = value.downcast_ref() {
                    if !self.is_sym_assigned(name) {
                        let typeset = self.execute_expr(typeset);

                        // type def
                        if let Some(set) = typeset.downcast_ref::<Rc<CanonSet>>() {
                            self.insert_sym_type(name.to_owned(), Rc::clone(set));
                            return;
                        } else {
                            panic!("'{typeset}' is not a set")
                        }
                    }
                }

                // type cast
                println!("{}", todo!());
            
            // } else if let Some(FuncTypeExpr(value, domain_arg_t, codomain)) = expr.downcast_ref() {
            //     if let Some(Symbol(name)) = value.downcast_ref() {
            //         if !self.is_sym_assigned(name) {
            //             let domain = self.execute_expr(&domain_arg_t[0]);
            //             let codomain = self.execute_expr(codomain);

            //             if let Some(domset) = domain.downcast_ref::<Rc<CanonSet>>() {
            //                 self.insert_sym_type(name, set);
            //             }
            //         }
            //     }

            //     // function type cast (perhaps for classes?)
            //     // eg maybe this?: f : impl [+ : A, A -> A], A -> A; f(a1, a2) = a1+a2; g : Int -> Int; g(x) = f : Int -> Int;
            //     println!("{}", todo!())
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
            if let Some(SymStore::Value(value)) = self.symbols.get(name) {
                value.clone()
            } else {
                panic!("Variable '{name}' is not defined");
            }
        } else if let Some(Group(expr)) = expr.downcast_ref::<Group>() {
            self.execute_expr(expr)
        } else if let Some(Unary(op, right)) = expr.downcast_ref() {
            let right = self.execute_expr(right);

            match op.kind() {
                &TokenKind::Minus => Self::execute_neg(&right),
                _ => todo!()
            }
        } else if let Some(Binary(left, op, right)) = expr.downcast_ref() {
            let left = self.execute_expr(left);
            let right = self.execute_expr(right);

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
        } else if let Some(func @ expr::Func(_, _)) = expr.downcast_ref() {
            Box::new(Func::from(func))
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

    fn execute_assign(&mut self, name: &str, right: &Box<dyn Expr>) {
        if self.is_sym_assigned(name) {
            panic!("Variable {name} cannot be reassigned")
        }

        let right = self.execute_expr(right);

        if let Some(_) = right.downcast_ref::<Func>() {
            self.insert_sym(
                name.to_owned(),
                right
            );

            return;
        }
        
        if let Some(SymStore::Type(typeset)) = self.symbols.get(name) {
            if !typeset.contains(&right) {
                panic!("'{name}' is in '{typeset}' which does not contain '{right}'")
            }
        }

        self.insert_sym(
            name.to_owned(),
            right
        );
    }

    fn execute_typed_assign(&mut self, name: &str, typeset: &Box<dyn Expr>, right: &Box<dyn Expr>) {
        if self.is_sym_assigned(name) {
            panic!("Variable '{name}' cannot be reassigned")
        }

        let typeset = self.execute_expr(typeset);

        if let Some(set) = typeset.downcast_ref::<Rc<CanonSet>>() {
            let value = self.execute_expr(right);

            if set.contains(&value) {
                self.insert_sym(name.to_owned(), value);
            } else {
                panic!("Incompatible types: '{value}' cannot be cast into '{typeset}'");
            }
        } else {
            panic!("'{typeset}' is not a set");
        }
    }
}
