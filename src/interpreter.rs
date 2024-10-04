use std::ops::Deref;

use num::{BigInt, BigRational, Complex};

use crate::ast::{Ast, expr::*, stmt::*};
use crate::token::{Token, TokenKind};
use crate::value::Val;

pub struct Interpreter<'t> {
    stmts: &'t [Box<dyn Stmt>] 
}

impl<'t> Interpreter<'t> {
    pub fn new(stmts: &'t [Box<dyn Stmt>]) -> Self {
        Self {
            stmts
        }
    }

    pub fn interpret(&mut self) {
        for stmt in self.stmts {
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

    pub fn execute_expr<'a>(&mut self, expr: &'a Box<dyn Expr>) -> &'a Box<dyn Val> {
        if let Some(Literal(lit)) = expr.downcast_ref() {
            if let Some(_) = lit.downcast_ref::<BigInt>() {
                lit
            } else if let Some(_) = lit.downcast_ref::<BigRational>() {
                lit
            } else if let Some(_) = lit.downcast_ref::<Complex<BigRational>>() {
                lit
            } else if let Some(_) = lit.downcast_ref::<String>(){
                lit
            } else if let Some(_) = lit.downcast_ref::<bool>() {
                lit
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }
}
