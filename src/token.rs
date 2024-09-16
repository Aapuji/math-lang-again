use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    line: usize
}

impl Token {
    pub fn new(kind: TokenKind, line: usize) -> Self {
        Self { kind, line }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub fn lexeme(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Ident(lexeme)    |
            TokenKind::Keyword(lexeme)  |
            TokenKind::String(lexeme)   |
            TokenKind::Char(lexeme)     |
            TokenKind::Number(lexeme)   => Some(lexeme),
            _ => None
        }
    }

    pub fn append_to_lexeme(&mut self, ch: char) {
        match self.kind {
            TokenKind::Ident(ref mut lexeme)    |
            TokenKind::Keyword(ref mut lexeme)  |
            TokenKind::String(ref mut lexeme)   |
            TokenKind::Char(ref mut lexeme)     |
            TokenKind::Number(ref mut lexeme)   => lexeme.push(ch),
            _ => ()
        };
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            kind: TokenKind::EOL,
            line: 0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Single-Character Tokens
    Plus, Minus, Star, Slash, Caret, Eq, Bang, Tilde, 
    Bar, Amp, BackSlash, Less, Greater, 
    OpenParen, CloseParen, OpenBracket, CloseBracket, 
    OpenBrace, CloseBrace, 
    Hash, Dot, Comma, Semicolon, Colon,
    Underscore,

    // Multi-Character Tokens
    DblEq, BangEq, LessEq, GreaterEq,

    // Value Tokens
    Ident(String), Keyword(String), String(String), Char(String), 
    Number(String), 

    EOL, EOF
}
