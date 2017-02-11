extern crate rustyline;

use std::fs::File;
use std::io::Read;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub mod ast;
pub mod lexer;
pub mod parser;

use lexer::Lexer;

fn main() {
    let mut args = std::env::args();
    match args.len() {
        1 => {
            let mut rl = Editor::<()>::new();
            loop {
                let readline = rl.readline("prf> ");
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(&line);
                        for tok in Lexer::new(&line).with_ws_stx() {
                            println!("{:?}", tok);
                        }
                        println!("");
                        println!("1: {:?}", parser::parse_Expr(Lexer::new(&line).with_ws_stx()));
                    },
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                    Err(err) => {
                        println!("Error: {:?}", err);
                        break;
                    }
                }
            }
        },
        2 => {
            let _ = args.next();
            let mut f = File::open(args.next().unwrap()).expect("unable to open file");
            let mut code = String::new();
            f.read_to_string(&mut code).expect("error reading from file");
            for tok in Lexer::new(&code).with_ws_stx() {
                println!("{:?}", tok);
            }
            println!("");
            println!("1: {:?}", parser::parse_Exprs(Lexer::new(&code).with_ws_stx()));
        },
        _ => println!("Too many command line arguments.")
    }
}
