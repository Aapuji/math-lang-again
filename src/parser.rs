use num::bigint::Sign;
use num::{BigInt, BigRational};

use crate::ast::{Ast, Expr, Stmt};
use crate::token::{Token, TokenKind};
use crate::types::{TNum, Type};
use crate::value::{Val, Value};

pub struct Parser<'t> {
    tokens: &'t [Token],
    line: usize,
    i: usize
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token]) -> Self {
        Self { 
            tokens, 
            line: 0,
            i: 0
        }
    }

    pub fn parse(&mut self) -> Ast {
        let mut ast = Ast::new();

        while self.current().kind() != &TokenKind::EOF {
            ast.push_stmt(self.parse_stmt());

        }

        ast
    }

    fn parse_stmt(&mut self) -> Stmt {
        let res = self.parse_declaration();
        
        self.next();
        
        if let &TokenKind::EOL = self.current().kind() {
            self.next();
        }

        res
    }

    fn parse_declaration(&mut self) -> Stmt {
        // if self.match_next(&TokenKind::Ident(_)) {

        // }

        Stmt::Expr(self.parse_expr())
    }

    fn parse_expr(&mut self) -> Expr {

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current().kind() {
            TokenKind::Number(l1) => if let Some(expr) = self.parse_number(l1.clone()) {
                return expr;
            }
            _ => ()
        }

        panic!("Expected expression") // Todo: change for actual error handling
    }

    fn parse_number(&mut self, mut l1: String) -> Option<Expr> {
        self.next();

        // Decimal (eg. 12.34)
        if let &TokenKind::Dot = self.current().kind() {
            fn getlen(s: &str) -> BigInt {
                let mut n = BigInt::from(1);

                for _ in s.chars() {
                    n *= 10;
                }

                n
            }

            if let Some(t) = self.peek() {
                if let TokenKind::Number(l2) = t.kind() {
                    let l2 = l2.clone();
                    l1.push_str(&l2);
                    self.next();

                    Some(Expr::Literal(
                        Value::new(
                            Val::Decimal(
                                BigRational::new(l1.parse().unwrap(), getlen(&l2))),
                            Type::Num(TNum::real())
                        )
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        // Int (eg. 1234)
        } else {
            Some(Expr::Literal(
                Value::new(
                    Val::Int(l1.parse().unwrap()), 
                    Type::Num(TNum::int())
            )))
        }
    }

    /// Consumes next token if it matches the given [`TokenKind`]
    fn match_next(&mut self, kind: &TokenKind) -> bool {
        if self.i >= self.tokens.len() {
            false
        } else if let Some(t) = self.peek() {
            dbg!("gogog");
            if t.kind() == kind {
                self.next();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.i + 1 > self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.i + 1])
        }
    }


    fn next(&mut self) {
        self.i += 1;
    }

    fn current(&self) -> &Token {
        &self.tokens[self.i]
    }
}
