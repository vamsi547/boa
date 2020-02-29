#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

pub mod builtins;
pub mod environment;
pub mod exec;
pub mod realm;
pub mod syntax;
#[cfg(feature = "wasm-bindgen")]
mod wasm;

#[cfg(feature = "wasm-bindgen")]
pub use crate::wasm::*;

use crate::{
    builtins::value::ResultValue,
    exec::{Executor, Interpreter},
    realm::Realm,
    syntax::{
        ast::{expr::Expr, token::Token},
        lexer::Lexer,
        parser::Parser,
    },
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use std::thread;

fn parser_expr(src: &str) -> Result<Expr, String> {
    let mut lexer = Lexer::new(src, None);
    lexer.lex().map_err(|e| format!("SyntaxError: {}", e))?;
    let tokens = lexer.tokens;
    Parser::new(tokens)
        .parse_all()
        .map_err(|e| format!("ParsingError: {}", e))
}

fn parser_expr_concurrent(src: &str) -> Result<Expr, String> {
    // Create channel to send tokens from the lexer to the parser
    let (tokenStreamSender, tokenStreamReceiver): (Sender<Token>, Receiver<Token>) = unbounded();

    thread::spawn(move || {
        let mut lexer = Lexer::new(src, Some(tokenStreamSender));
        lexer
            .lex()
            .map_err(|e| format!("SyntaxError: {}", e))
            .unwrap();
    });

    // Parser::new(tokens)
    //     .parse_all()
    //     .map_err(|e| format!("ParsingError: {}", e))
}

/// Execute the code using an existing Interpreter
/// The str is consumed and the state of the Interpreter is changed
pub fn forward(engine: &mut Interpreter, src: &str) -> String {
    // Setup executor
    let expr = match parser_expr(src) {
        Ok(v) => v,
        Err(error_string) => {
            return error_string;
        }
    };
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => format!("{}: {}", "Error", v.to_string()),
    }
}

/// Execute the code using an existing Interpreter.
/// The str is consumed and the state of the Interpreter is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
pub fn forward_val(engine: &mut Interpreter, src: &str) -> ResultValue {
    // Setup executor
    match parser_expr(src) {
        Ok(expr) => engine.run(&expr),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    // Create new Realm
    let realm = Realm::create();
    let mut engine: Interpreter = Executor::new(realm);
    forward(&mut engine, src)
}
