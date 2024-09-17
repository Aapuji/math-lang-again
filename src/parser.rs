use crate::ast::Ast;
use crate::token::{Token, TokenKind};

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

        while self.tokens[self.i].kind() != &TokenKind::EOF {
            self.parse_declaration(&mut ast);
        }

        todo!()
    }

    fn parse_declaration(&mut self, ast: &mut Ast) {
        if self.match_next(&TokenKind::Ident(_)) {

        }
    }

    /// Consumes next token if it matches the given [`TokenKind`]
    fn match_next(&mut self, kind: &TokenKind) -> bool {
        if self.i >= self.tokens.len() {
            false
        } else {
            matches!(self.tokens[self.i + 1].kind(), kind)
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.i]
    }
}
