use crate::{token::Token, value::Val};

#[derive(Debug)]
pub struct Ast {
    stmts: Vec<Stmt>
}

impl Ast {
    pub fn new() -> Self {
        Self {
            stmts: vec![]
        }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr)
}

#[derive(Debug)]
pub enum Expr {
    Literal(Box<dyn Val>),
    Group(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Assign(String, Box<Expr>),
    Func(Vec<String>, Box<Expr>),
    List(Vec<Expr>),
    Matrix(Vec<Vec<Expr>>)
}
