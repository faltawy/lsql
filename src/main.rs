// LSQL - A command-line utility for querying files and directories using SQL-like syntax
// This is the main entry point for the application

mod cli;
mod display;
mod filter;
mod fs;
mod parser;
mod shell;
mod theme;

use clap::Parser;
use cli::{Args, CLI};
use log::{error, info};

fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Create CLI instance (this will setup the logger)
    let cli = CLI::new(args.clone());

    info!("LSQL started");

    // Check if interactive mode is enabled, or Shell subcommand is used
    let is_interactive = args.interactive || matches!(args.command, Some(cli::Command::Shell));

    if is_interactive {
        // Run in interactive shell mode
        let mut shell = shell::LSQLShell::new();
        shell.run(&cli);
    } else {
        // Run in normal mode with command line arguments
        if let Err(e) = cli.run(args) {
            error!("Error: {}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    info!("LSQL completed successfully");
}
