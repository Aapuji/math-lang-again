mod ast;
mod config;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod set;
mod token;
mod types;
mod value;

use std::env;

use config::{Config, Mode};
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let config = Config::build(env::args()).unwrap();

    if let Mode::File(path) = config.mode() {
        let stuff = std::fs::File::open(path).unwrap();
        let mut lexer = Lexer::new(stuff);

        let tokens = lexer.lex().unwrap();

        // println!("{:#?}", tokens);

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();

        println!("{:#?}", ast);

        let mut interpreter = Interpreter::new();
        interpreter.interpret(ast.stmts());
        println!("{:#?}", interpreter);
    }
}
