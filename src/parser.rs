use num::bigint::Sign;
use num::{BigInt, BigRational, Complex, FromPrimitive};

use crate::ast::{Ast, Expr, Stmt};
use crate::token::{Token, TokenKind};
use crate::types::{TNum, TText, Type};
use crate::value::{Val, Value};

pub struct Parser<'t> {
    tokens: &'t [Token],
    line: usize,
    i: usize
}

impl<'t> Parser<'t> {
    const KEYWORDS: [&'static str; 7] = ["do", "end", "data", "class", "mod", "import", "as"];

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
                
        if let &TokenKind::EOL = self.current().kind() {
            self.next();
        }

        res
    }

    fn parse_declaration(&mut self) -> Stmt {
        // if self.match_next(&TokenKind::Ident(_)) {

        // }

        Stmt::Expr({ let x = self.parse_expr(); self.next(); x })
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();

        while self.match_next(&[&TokenKind::Plus, &TokenKind::Minus]) {
            let op = self.current().clone();
            self.next();
            let right = self.parse_factor();

            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        return expr;
    }

    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_unary();

        while self.match_next(&[&TokenKind::Slash, &TokenKind::Star]) {
            let op = self.current().clone();
            self.next();
            let right = self.parse_unary();

            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        return expr;
    }

    fn parse_unary(&mut self) -> Expr {
        match self.current().kind() {
            TokenKind::Bang  |
            TokenKind::Minus |
            TokenKind::Plus  |
            TokenKind::Tilde => {
                let op = self.current().clone();
                self.next();
                let right = self.parse_unary();

                return Expr::Unary(op, Box::new(right));
            }
            _ => self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        if self.match_next(&[&TokenKind::Caret]) {
            let op = self.current().clone();

            self.next();
            let right = self.parse_unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        } else {
            return expr;
        }

        expr 
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current().kind() {
            TokenKind::Ident(lexeme) => return self.parse_ident(lexeme.clone()),
            TokenKind::String(lexeme) => return self.parse_string(lexeme.clone()),
            TokenKind::Number(lexeme) => return self.parse_number(lexeme.clone()),
            TokenKind::OpenParen => return self.parse_grouping(),

            _ => ()
        }

        panic!("Expected expression") // Todo: change for actual error handling
    }

    // Parsing of keywords will have already been done prior, in the statement parsing parts
    fn parse_ident(&mut self, lexeme: String) -> Expr {
        if &lexeme == "i" {
            Expr::Literal(
                Value::new(
                    Val::Complex(Complex::new(BigRational::from(BigInt::from(0)), BigRational::from(BigInt::from(1)))),
                    Type::Num(TNum::complex())
                )
            )
        } else {
            Expr::Literal(
                Value::new(
                    Val::Ident(lexeme),
                    Type::Symbol
                )
            )
        }
    }

    fn parse_string(&mut self, lexeme: String) -> Expr {
        Expr::Literal(
            Value::new(
                Val::String(lexeme), 
                Type::Text(TText::str())
            )
        )
    }

    fn parse_number(&mut self, mut l1: String) -> Expr {
        // Decimal (eg. 12.34)
        let mut num = if self.match_next(&[&TokenKind::Dot]) {
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

                    Expr::Literal(
                        Value::new(
                            Val::Decimal(
                                BigRational::new(l1.parse().unwrap(), getlen(&l2))),
                            Type::Num(TNum::real())
                        )
                    )
                } else {
                    Expr::Literal(
                        Value::new(
                            Val::Int(l1.parse().unwrap()), 
                            Type::Num(TNum::int())
                    ))
                }
            } else {
                Expr::Literal(
                    Value::new(
                        Val::Int(l1.parse().unwrap()), 
                        Type::Num(TNum::int())
                ))
            }
        // Int (eg. 1234)
        } else {
            Expr::Literal(
                Value::new(
                    Val::Int(l1.parse().unwrap()), 
                    Type::Num(TNum::int())
            ))
        };


        if self.match_next(&[&TokenKind::Ident("i".to_owned())]) {
            let val = match num {
                Expr::Literal(v) => v,
                _ => unreachable!()
            };
            
            let new_val = match val.val_move() {
                Val::Int(x) => Val::Complex(Complex::new(BigRational::from(BigInt::from(0)), x.into())),
                Val::Decimal(x) => Val::Complex(Complex::new(BigRational::from(BigInt::from(0)), x.into())),
                _ => unreachable!()
            };

            num = Expr::Literal(Value::new(new_val, Type::Num(TNum::complex())));
        }

        num
    }

    fn parse_grouping(&mut self) -> Expr {
        self.next();

        let expr = self.parse_expr();
        
        dbg!(&self.tokens[self.i..]);
        if self.match_next(&[&TokenKind::CloseParen]) {
            ()
        } else {
            panic!("Closing parenthesis expected");
        }

        Expr::Group(Box::new(expr))
    }

    /// Consumes next token if it matches the given [`TokenKind`]
    fn match_next(&mut self, kind: &[&TokenKind]) -> bool {
        if self.i >= self.tokens.len() {
            false
        } else if let Some(t) = self.peek() {
            if  kind.iter().any(|&k| t.kind() == k) {
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
