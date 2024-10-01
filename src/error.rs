use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Lexer(LexError)
}

#[derive(Debug)]
pub enum LexError {
    UnclosedString,
    UnclosedParenthesis,
    UnclosedBracket
}

pub type Result<T> = core::result::Result<T, Error>;
