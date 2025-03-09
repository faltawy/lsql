// CLI Module
// This module handles command-line arguments, interactive shell, and user interface

use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use log::{debug, error, info, warn};
use rustyline::{error::ReadlineError, DefaultEditor};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};

use crate::display;
use crate::fs;
use crate::parser::{LSQLParser, SelectionType};

// Define the log level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// Define the CLI arguments structure
#[derive(Parser, Clone)]
#[clap(
    name = "lsql",
    about = "Query files and directories using SQL-like syntax",
    version
)]
pub struct Args {
    /// SQL-like query to execute
    #[clap(index = 1)]
    query: Option<String>,

    /// Disable colored output
    #[clap(long, short = 'n')]
    no_color: bool,

    /// Enable recursive search (default is non-recursive)
    #[clap(long, short = 'r')]
    recursive: bool,

    /// Set the logging level
    #[clap(long, short = 'l', value_enum, default_value = "info")]
    log_level: LogLevel,

    /// Subcommands
    #[clap(subcommand)]
    command: Option<Command>,
}

// Define subcommands
#[derive(Subcommand, Clone)]
enum Command {
    /// Start interactive shell
    Shell,
}

// Input helper for rustyline
#[derive(Helper, Completer, Hinter, Validator, Highlighter)]
struct InputHelper;

// Main CLI handler
pub struct CLI {
    use_color: bool,
    recursive: bool,
}

impl CLI {
    // Create a new CLI instance from args
    pub fn new(args: Args) -> Self {
        // Setup logger with appropriate level
        Self::setup_logger(args.log_level);

        // Check if color should be enabled
        let use_color = !args.no_color;

        CLI {
            use_color,
            recursive: args.recursive,
        }
    }

    // Setup the logger based on the provided log level
    fn setup_logger(level: LogLevel) {
        let filter_level = match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        };

        env_logger::Builder::new()
            .filter_level(filter_level)
            .format_timestamp(None)
            .init();

        debug!("Logger initialized with level: {:?}", level);
    }

    // Run the CLI
    pub fn run(self, args: Args) -> Result<(), String> {
        debug!(
            "Running with args: recursive={}, color={}",
            self.recursive, self.use_color
        );

        match args.command {
            Some(Command::Shell) => self.run_interactive_shell(),
            None => match args.query {
                Some(query) => self.execute_query(&query),
                None => {
                    info!("No query provided. Use --help for usage information or 'lsql shell' for interactive mode.");
                    println!("No query provided. Use --help for usage information or 'lsql shell' for interactive mode.");
                    Ok(())
                }
            },
        }
    }

    // Run interactive shell
    fn run_interactive_shell(&self) -> Result<(), String> {
        debug!("Starting interactive shell");

        let mut rl = DefaultEditor::new().map_err(|e| format!("Failed to start editor: {}", e))?;
        let _ = rl.load_history(".lsql_history"); // Ignore error if no history file

        // Print welcome message
        println!("Welcome to LSQL Shell. Type SQL-like queries to explore your filesystem.");
        println!("Examples:");
        println!("  select * from .;");
        println!("  select files from ./Downloads where ext=\"pdf\";");
        println!("  select * from . where size > \"10mb\";");
        println!("Type 'exit' or press Ctrl+D to exit.");
        println!();

        loop {
            let prompt = if self.use_color {
                "lsql> ".green().to_string()
            } else {
                "lsql> ".to_string()
            };

            match rl.readline(&prompt) {
                Ok(line) => {
                    rl.add_history_entry(&line)
                        .map_err(|e| format!("History error: {}", e))?;

                    let line = line.trim();
                    if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
                        debug!("User requested exit from shell");
                        break;
                    }

                    if !line.is_empty() {
                        debug!("Executing query: {}", line);
                        if let Err(e) = self.execute_query(line) {
                            error!("Query execution failed: {}", e);
                            if self.use_color {
                                eprintln!("{}", format!("Error: {}", e).red());
                            } else {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C, ignore
                    debug!("Readline interrupted (Ctrl+C)");
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D
                    debug!("Readline EOF (Ctrl+D)");
                    break;
                }
                Err(e) => {
                    error!("Readline error: {}", e);
                    if self.use_color {
                        eprintln!("{}", format!("Error: {}", e).red());
                    } else {
                        eprintln!("Error: {}", e);
                    }
                    break;
                }
            }
        }

        info!("Shell session ended");
        let _ = rl.save_history(".lsql_history"); // Ignore error on save
        Ok(())
    }

    // Execute a single query
    fn execute_query(&self, query_str: &str) -> Result<(), String> {
        debug!("Parsing query: {}", query_str);

        // Parse the query
        let query = match LSQLParser::parse_query(query_str) {
            Ok(q) => q,
            Err(e) => {
                error!("Query parsing failed: {}", e);
                return Err(e);
            }
        };

        debug!("Query parsed successfully: {:?}", query);

        // Get filesystem entries
        let entries = match fs::list_entries(
            &query.path,
            &query.selection,
            &query.condition,
            self.recursive,
        ) {
            Ok(entries) => entries,
            Err(e) => {
                error!("Failed to list entries: {}", e);
                return Err(e);
            }
        };

        debug!("Found {} entries before filtering", entries.len());

        // Filter entries based on selection type
        let filtered_entries = entries
            .into_iter()
            .filter(|entry| match query.selection {
                SelectionType::All => true,
                SelectionType::Files => entry.is_file,
                SelectionType::Directories => entry.is_dir,
                SelectionType::Fields(_) => true,
            })
            .collect::<Vec<_>>();

        debug!("Filtered to {} entries", filtered_entries.len());

        // Display results
        if filtered_entries.is_empty() {
            info!("No results found for query");
            if self.use_color {
                println!("{}", "No results found.".yellow());
            } else {
                println!("No results found.");
            }
        } else {
            let table =
                display::display_entries(&filtered_entries, &query.selection, self.use_color);
            println!("{}", table);

            let count_message = format!(
                "{} items found{}",
                filtered_entries.len(),
                if self.recursive {
                    " (recursive search)"
                } else {
                    ""
                }
            );

            info!("{}", count_message);

            if self.use_color {
                println!(
                    "{} {} {}",
                    filtered_entries.len().to_string().green(),
                    "items found.".green(),
                    if self.recursive {
                        "(recursive search)".dimmed()
                    } else {
                        "".normal()
                    }
                );
            } else {
                println!(
                    "{} items found.{}",
                    filtered_entries.len(),
                    if self.recursive {
                        " (recursive search)"
                    } else {
                        ""
                    }
                );
            }
        }

        Ok(())
    }
}
