mod config;
mod lexer;
mod token;

use std::env;

use config::{Config, Mode};
use lexer::Lexer;

fn main() {
    let config = Config::build(env::args()).unwrap();

    if let Mode::File(path) = config.mode() {
        let stuff = std::fs::File::open(path).unwrap();
        let mut lexer = Lexer::new(stuff);

        let tokens = lexer.lex().unwrap();

        println!("{:#?}", tokens);
    }

}
