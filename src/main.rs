// LSQL - A command-line utility for querying files and directories using SQL-like syntax
// This is the main entry point for the application

mod cli;
mod display;
mod fs;
mod parser;

use clap::Parser;
use cli::{Args, CLI};
use log::{error, info};

fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Create CLI instance (this will setup the logger)
    let cli = CLI::new(args.clone());

    info!("LSQL started");

    if let Err(e) = cli.run(args) {
        error!("Error: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    info!("LSQL completed successfully");
}
