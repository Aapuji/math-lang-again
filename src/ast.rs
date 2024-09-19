use crate::value::Value;

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
    Literal(Value),
    Group(Box<Expr>)
}
