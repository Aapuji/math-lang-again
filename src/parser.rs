use num::bigint::Sign;
use num::{BigInt, BigRational, Complex};

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
            TokenKind::Ident(lexeme) => return self.parse_ident(lexeme.clone()),
            TokenKind::String(lexeme) => return self.parse_string(lexeme.clone()),
            TokenKind::Number(lexeme) => return self.parse_number(lexeme.clone()),
            _ => ()
        }

        dbg!(&self.tokens[self.i..]);
        panic!("Expected expression") // Todo: change for actual error handling
    }

    // Parsing of keywords will have already been done prior, in the statement parsing parts
    fn parse_ident(&mut self, lexeme: String) -> Expr {
        Expr::Literal(
            Value::new(
                Val::Ident(lexeme),
                Type::Symbol
            )
        )
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
        let mut num = if self.match_next(&TokenKind::Dot) {
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


        if self.match_next(&TokenKind::Ident("i".to_owned())) {
            let Expr::Literal(val) = num;
            
            let new_val = match val.val_move() {
                Val::Int(x) => Val::Complex(Complex::new(BigRational::from(BigInt::from(0)), x.into())),
                Val::Decimal(x) => Val::Complex(Complex::new(BigRational::from(BigInt::from(0)), x.into())),
                _ => unreachable!()
            };

            num = Expr::Literal(Value::new(new_val, Type::Num(TNum::complex())));
        }

        num
    }

    /// Consumes next token if it matches the given [`TokenKind`]
    fn match_next(&mut self, kind: &TokenKind) -> bool {
        if self.i >= self.tokens.len() {
            false
        } else if let Some(t) = self.peek() {
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
