mod ast;
mod config;
mod error;
mod lexer;
mod parser;
mod token;
mod types;
mod value;

use std::env;

use config::{Config, Mode};
use lexer::Lexer;
use parser::Parser;

fn main() {
    let config = Config::build(env::args()).unwrap();

    if let Mode::File(path) = config.mode() {
        let stuff = std::fs::File::open(path).unwrap();
        let mut lexer = Lexer::new(stuff);

        let tokens = lexer.lex().unwrap();

        println!("tokens: {:#?}", tokens);

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();

        println!("{:#?}", ast);
    }
}
