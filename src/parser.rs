use core::panic;
use num::{BigInt, BigRational, Complex, Zero};

use crate::ast::{Ast, expr::*, stmt::*};
use crate::token::{Token, TokenKind};

pub struct Parser<'t> {
    tokens: &'t [Token],
    line: usize,
    i: usize
}

impl<'t> Parser<'t> {
    const KEYWORDS: [&'static str; 8] = ["do", "end", "data", "class", "object", "import", "as", "proc"];

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
            if let TokenKind::EOL | TokenKind::Semicolon = self.current().kind() {
                if let TokenKind::EOL = self.current().kind() {
                    self.line += 1;
                }
                
                self.next();
                continue;
            }

            ast.push_stmt(self.parse_stmt());
        }

        ast
    }

    fn parse_stmt(&mut self) -> Box<dyn Stmt> {        
        let res = self.parse_expr_stmt();

        res
    }

    fn parse_expr_stmt(&mut self) -> Box<dyn Stmt> {
        let expr = self.parse_expr();

        if self.match_next(&[&TokenKind::EOL]) {
            // add log stmt here
            ()
        }

        if self.match_next(&[&TokenKind::Semicolon]) {
            ()
        }

        Box::new(ExprStmt(expr))
    }

    fn parse_expr(&mut self) -> Box<dyn Expr> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Box<dyn Expr> {
        let expr = self.parse_or();

        if self.match_next(&[&TokenKind::Eq]) {
            self.next();
            let right = self.parse_assign();

            // Parse func: f(x, y, ...) = expr
            if let Some(Call(left, args)) = expr.downcast_ref() {
                let name = if let Some(Symbol(name)) = left.downcast_ref() {
                    Some(name)
                } else {
                    None
                };

                if name.is_none() {
                    panic!("Invalid left-hand for function definition: f name");
                }
                
                let args = args
                    .into_iter()
                    .map(|a| {
                        if let Some(Symbol(arg)) = a.downcast_ref() {
                            return Symbol(arg.clone());
                        }

                        // args_are_syms = false;
                        panic!("Invalid left-hand for function definition: f args");
                        //String::new() // placeholder, won't be used if `args_are_syms` is `false`
                    })
                    .collect();

                let name = name.unwrap();
                
                return Box::new(Assign(Symbol(name.clone()), Box::new(Func(args, right))))
            // Parse var: x = expr
            } else if let Some(Symbol(name)) = expr.downcast_ref() {
                return Box::new(Assign(Symbol(name.clone()), right));
            }

            panic!("Invalid left-hand for assignment"); // TODO: Have actual error reporting
        }

        expr
    }

    fn parse_or(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_and();

        while self.match_next(&[&TokenKind::DblBar]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_and();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_and(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_comp();

        while self.match_next(&[&TokenKind::DblAmp]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_comp();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    // TODO: Have it allow for a < b < c.
    // Perhaps in another pass? As it will have to check if the type implements the Ord class rather than just PartialOrd.
    fn parse_comp(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_set_comp();

        while self.match_next(&[
            &TokenKind::DblEq, 
            &TokenKind::Less, &TokenKind::Greater,
            &TokenKind::LessEq, &TokenKind::GreaterEq
        ]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_set_comp();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_set_comp(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_set_ops();

        while self.match_next(&[
            &TokenKind::EqColon,
            &TokenKind::LessColon, &TokenKind::GreaterColon,
            &TokenKind::LessEqColon, &TokenKind::GreaterEqColon
        ]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_set_ops();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_set_ops(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_term();

        while self.match_next(&[&TokenKind::Amp, &TokenKind::Bar, &TokenKind::BackSlash]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_term();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_term(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_factor();

        while self.match_next(&[&TokenKind::Plus, &TokenKind::Minus]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_factor();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_factor(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_unary();

        while self.match_next(&[&TokenKind::Slash, &TokenKind::Star]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_unary();

            expr = Box::new(Binary(expr, op, right));
        }

        expr
    }

    fn parse_unary(&mut self) -> Box<dyn Expr> {
        match self.current().kind() {
            TokenKind::Bang  |
            TokenKind::Minus |
            TokenKind::Plus  |
            TokenKind::Tilde => {
                let op = self.current().clone();
                self.next();

                let right = self.parse_unary();

                return Box::new(Unary(op, right));
            }
            _ => self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_call();

        if self.match_next(&[&TokenKind::Caret]) {
            let op = self.current().clone();
            self.next();

            let right = self.parse_power();
            expr = Box::new(Binary(expr, op, right));
        }

        expr 
    }

    fn parse_call(&mut self) -> Box<dyn Expr> {
        let mut expr = self.parse_primary();

        loop {
            if self.match_next(&[&TokenKind::OpenParen]) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Box<dyn Expr>) -> Box<dyn Expr> {
        let mut args = vec![];

        if !matches!(self.current().kind(), &TokenKind::EOF) {
            // A do-while loop
            while {
                if !self.match_next(&[&TokenKind::Comma]) {
                    self.next();

                    args.push(self.parse_expr());

                    self.match_next(&[&TokenKind::Comma])
                } else {
                    true
                }
            } {}
        }

        if self.match_next(&[&TokenKind::CloseParen]) {
            ()
        } else {
            panic!("Expected ')' after arguments.");
        }

        Box::new(Call(callee, args))
    }

    fn parse_primary(&mut self) -> Box<dyn Expr> {
        return match self.current().kind() {
            TokenKind::Ident(lexeme) => self.parse_ident(lexeme.clone()),
            TokenKind::String(lexeme) => self.parse_string(lexeme.clone()),
            TokenKind::Char(lexeme) => self.parse_char(lexeme.clone()),
            TokenKind::Number(lexeme) => self.parse_number(lexeme.clone()),
            TokenKind::OpenParen => self.parse_grouping(),
            TokenKind::OpenBracket => self.parse_list(),
            
            _ => panic!("Expected expression {:#?}", &self.tokens[self.i..]) // Todo: change for actual error handling
        }
    }

    fn parse_ident(&mut self, lexeme: String) -> Box<dyn Expr> {
        if &lexeme == "i" {
            Box::new(Literal(
                Box::new(
                    Complex::<BigRational>::new(
                        BigRational::from(BigInt::from(0)), 
                        BigRational::from(BigInt::from(1))
                    )
                )
            ))
        } else if lexeme == "true" {
            Box::new(Literal(Box::new(true)))
        } else if lexeme == "false" {
            Box::new(Literal(Box::new(false)))
        } else if let Some(&keyword) = Self::KEYWORDS.iter().find(|&k| k == &lexeme) {
            todo!()
            // Box::new(Keyword(keyword.to_owned()))
        } else {
            Box::new(Symbol(lexeme))
        }
    }

    fn parse_string(&mut self, lexeme: String) -> Box<dyn Expr> {
        Box::new(Literal(Box::new(lexeme)))
    }

    fn parse_char(&mut self, lexeme: String) -> Box<dyn Expr> {
        Box::new(Literal(Box::new(lexeme))) // todo, make `struct Char(String)` struct to hold chars
    }

    fn parse_number(&mut self, mut l1: String) -> Box<dyn Expr> {
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

                    Box::new(Literal(Box::new(BigRational::new(l1.parse().unwrap(), getlen(&l2)))))
                } else {
                    Box::new(Literal(Box::new(l1.parse::<BigInt>().unwrap())))
                }
            } else {
                Box::new(Literal(Box::new(l1.parse::<BigInt>().unwrap())))
            }
        // Int (eg. 1234)
        } else {
            Box::new(Literal(Box::new(l1.parse::<BigInt>().unwrap())))
        };


        if self.match_next(&[&TokenKind::Ident("i".to_owned())]) {
            let val = match *num {
                Literal(v) => v,
                _ => unreachable!()
            };
            
            let new_val = if let Some(bigint) = val.as_any().downcast_ref::<BigInt>() {
                Complex::<BigRational>::new(BigRational::zero(), BigRational::from(bigint.to_owned()))
            } else if let Some(bigrat) = val.as_any().downcast_ref::<BigRational>() {
                Complex::<BigRational>::new(BigRational::zero(), bigrat.to_owned())
            } else {
                unreachable!()
            };

            num = Box::new(Literal(Box::new(new_val)));
        }

        num
    }

    fn parse_grouping(&mut self) -> Box<dyn Expr> {
        self.next();

        let expr = self.parse_expr();
        
        if self.match_next(&[&TokenKind::CloseParen]) {
            ()
        } else {
            panic!("Closing parenthesis expected");
        }

        Box::new(Group(expr))
    }

    fn parse_list(&mut self) -> Box<dyn Expr> {
        self.next();

        let mut matrix_dim = None;
        let mut result = Vec::new();
        let mut list = Vec::new();

        while self.current().kind() != &TokenKind::CloseBracket {
            list.push(self.parse_expr());

            if self.match_next(&[&TokenKind::Comma]) {
                self.next();

                continue
            // If sees semicolon, it creates matrix instead of list (tuple)
            } else if self.match_next(&[&TokenKind::Semicolon]) {
                if let None = matrix_dim {
                    matrix_dim = Some((1usize, list.len()));
                } else if let Some((r, c)) = matrix_dim {
                    if c != list.len() {
                        panic!("Each row of a matrix must have the same length: 1");
                    }

                    matrix_dim = Some((r + 1, c));
                }
                
                result.push(list);
                list = Vec::new();

                self.next();
                continue;
            } else if self.match_next(&[&TokenKind::CloseBracket]) {
                if let Some((r, c)) = matrix_dim {
                    if c != list.len() {
                        panic!("Each row of a matrix must have the same length: 2");
                    }
                    
                    matrix_dim = Some((r + 1, c));
                    result.push(list);
                    list = Vec::new();
                }
            } else if self.match_next(&[&TokenKind::EOF]) {
                panic!("Expected ']'.");
            } else {
                panic!("Expected ',', ';', or ']'.");
            }
        }

        if let Some(_) = matrix_dim {
            Box::new(Matrix(result))
        } else {
            Box::new(Tuple(list))
        }
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
