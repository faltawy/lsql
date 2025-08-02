use clap::Parser;

use crate::cli::{Args, Cli, Command};
use crate::shell::LSQLShell;
use log::{error, info};

/// High level interpreter coordinating CLI, parser, filesystem and display layers.
pub struct Interpreter;

impl Interpreter {
    /// Run the interpreter using command line arguments.
    pub fn run() -> Result<(), String> {
        // Parse command line arguments
        let args = Args::parse();

        // Initialize the CLI (sets up logging and themes)
        let cli = Cli::new(args.clone());

        info!("LSQL started");

        let is_interactive = args.interactive || matches!(args.command, Some(Command::Shell));

        if is_interactive {
            let mut shell = LSQLShell::new();
            shell.run(&cli);
            info!("LSQL completed successfully");
            Ok(())
        } else {
            let res = cli.run(args);
            if let Err(ref e) = res {
                error!("Error: {}", e);
                eprintln!("Error: {}", e);
            } else {
                info!("LSQL completed successfully");
            }
            res
        }
    }
}
