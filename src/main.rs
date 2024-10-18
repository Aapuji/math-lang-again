mod ast;
mod config;
mod error;
mod interpreter;
mod iter;
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

        println!("\n--- Tokens ---\n{:#?}", tokens);

        let mut parser = Parser::new(&tokens);
        let ast = parser.parse();

        println!("\n--- AST---\n{:#?}", ast);

        println!("\n--- Code Output ---");

        let mut interpreter = Interpreter::new();
        interpreter.interpret(ast.stmts());

        println!("\n--- Interpreter State ---\n{:#?}", interpreter);
    }
}
