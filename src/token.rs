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

    pub fn lexeme(&self) -> &str {
        match &self.kind {
            TokenKind::Ident(lexeme)    |
            TokenKind::String(lexeme)   |
            TokenKind::Char(lexeme)     |
            TokenKind::Number(lexeme)   => lexeme,
            
            TokenKind::Amp => "&",
            TokenKind::BackSlash => "\\",
            TokenKind::Bang => "!",
            TokenKind::BangEq => "!=",
            TokenKind::Bar => "|",
            TokenKind::Caret => "^",
            TokenKind::CloseBrace => "}",
            TokenKind::CloseBracket => "]",
            TokenKind::CloseParen => ")",
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::DblAmp => "&&",
            TokenKind::DblBar => "||",
            TokenKind::DblEq => "==",
            TokenKind::Dot => ".",
            TokenKind::Eq => "=",
            TokenKind::EqColon => "=:",
            TokenKind::FatArrow => "=>",
            TokenKind::Greater => ">",
            TokenKind::GreaterColon => ">:",
            TokenKind::GreaterEq => ">=",
            TokenKind::GreaterEqColon => ">=:",
            TokenKind::Hash => "#",
            TokenKind::Less => "<",
            TokenKind::LessColon => "<:",
            TokenKind::LessEq => "<=",
            TokenKind::LessEqColon => "<=:",
            TokenKind::Minus => "-",
            TokenKind::OpenBrace => "{",
            TokenKind::OpenBracket => "[",
            TokenKind::OpenParen => "(",
            TokenKind::Plus => "+",
            TokenKind::Semicolon => ";",
            TokenKind::Slash => "/",
            TokenKind::SmallArrow => "->",
            TokenKind::Star => "*",
            TokenKind::Tilde => "~",

            TokenKind::EOF | 
            TokenKind::EOL => ""
        }
    }

    pub fn append_to_lexeme(&mut self, ch: char) {
        match self.kind {
            TokenKind::Ident(ref mut lexeme)    |
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

    // Double-Character Tokens
    DblEq, BangEq, LessEq, GreaterEq,
    DblAmp, DblBar,
    EqColon, LessColon, GreaterColon, 
    SmallArrow, FatArrow,

    
    // Triple-Character Tokens
    LessEqColon, GreaterEqColon,

    // Value Tokens
    Ident(String), String(String), Char(String),
    Number(String), 

    EOL, EOF
}
