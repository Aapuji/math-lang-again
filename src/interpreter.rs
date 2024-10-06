use std::collections::HashMap;
use num::{BigInt, BigRational, Complex};

use crate::ast::{expr::*, stmt::*};
use crate::token::TokenKind;
use crate::value::{Tuple, Val};

struct State {
    symbols: HashMap<String, Box<dyn Val>>
}

impl State {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        map.insert(String::from("arr"), Box::new(BigInt::from(1)));

        Self {
            symbols: HashMap::new()
        }
    }
}

pub struct Interpreter {
    state: State
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            state: State::new()
        }
    }

    pub fn interpret<'t>(&mut self, stmts: &'t [Box<dyn Stmt>]) {
        for stmt in stmts {
            self.execute_stmt(stmt);
        }
    }

    pub fn execute_stmt(&mut self, stmt: &Box<dyn Stmt>) {
        if let Some(ExprStmt(expr)) = stmt.downcast_ref() {
            println!("{}", self.execute_expr(expr))
        } else {
            todo!()
        }
    }

    pub fn execute_expr<'a>(&mut self, expr: &'a Box<dyn Expr>) -> Box<dyn Val> {
        if let Some(Literal(lit)) = expr.downcast_ref() {
            Self::execute_literal(lit)
        } else if let Some(Symbol(name)) = expr.downcast_ref() {
            if self.state.symbols.contains_key(name) {
                (*self.state.symbols.get(name).unwrap()).clone()
            } else {
                panic!("Variable {name} is not defined");
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
                _ => todo!()
            }
        // } else if let Some(expr::Tuple(exprs)) = expr.downcast_ref() {
        //     Box::new(Tuple(exprs
        //         .iter()
        //         .map(|expr| self.execute_expr(expr))
        //         .collect::<Vec<Box<dyn Val>>>()))
        } else if let Some(Assign(Symbol(name), right)) = expr.downcast_ref() {
            self.execute_assign(name, right)
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
        } else {
            panic!("Cannot apply unary operator '-'.");
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
                panic!("Cannot apply binary operator '+'.")
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
                panic!("Cannot apply binary operator '+'.")
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
                panic!("Cannot apply binary operator '+'.")
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
                panic!("Cannot apply binary operator '+'.")
            }
        } else {
            panic!("Cannot apply binary operator '+'.")
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
                panic!("Cannot apply binary operator '*'.")
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
                panic!("Cannot apply binary operator '*'.")
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
                panic!("Cannot apply binary operator '*'.")
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
                panic!("Cannot apply binary operator '*'.")
            }
        } else {
            panic!("Cannot apply binary operator '*'.")
        }
    }

    fn execute_quot(left: &Box<dyn Val>, right: &Box<dyn Val>) -> Box<dyn Val> {
        if left.is_str() || right.is_str() {
            panic!("Cannot apply binary operator '/' to text.")
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
                panic!("Cannot divide by a boolean.")
            } else {
                panic!("Cannot apply binary operator '/'.")
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
                panic!("Cannot apply binary operator '.'.")
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
                panic!("Cannot apply binary operator '/'.")
            }
        // Cannot use division with booleans
        } else if let Ok(_) = left.downcast::<bool>() {
            panic!("Cannot use division with booleans")
        } else {
            panic!("Cannot apply binary operator '/'.")
        }
    }

    // In future, return a result of whether it assigned or not?
    fn execute_assign(&mut self, name: &str, right: &Box<dyn Expr>) -> Box<dyn Val> {
        if self.state.symbols.contains_key(name) {
            panic!("Variables cannot be reassigned")
        } else {
            let right = self.execute_expr(right);
            self.state.symbols.insert(name.to_owned(), (*right).clone_box());

            (*self.state.symbols.get(name).unwrap()).clone()
        }
    }
}
