// LSQL - A command-line utility for querying files and directories using SQL-like syntax
// This is the main entry point for the application

mod cli;
mod display;
mod filter;
mod fs;
mod interpreter;
mod parser;
mod shell;
mod theme;

use interpreter::Interpreter;

fn main() {
    if let Err(_) = Interpreter::run() {
        std::process::exit(1);
    }
}
