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
use crate::theme::{apply_color, Theme, ThemeManager};

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

    /// Select the color theme
    #[clap(long, short = 't', default_value = "default")]
    theme: String,

    /// List available themes
    #[clap(long)]
    list_themes: bool,

    /// Subcommands
    #[clap(subcommand)]
    command: Option<Command>,
}

// Define subcommands
#[derive(Subcommand, Clone)]
enum Command {
    /// Start interactive shell
    Shell,

    /// Theme management
    Theme {
        /// Theme operation (list, create, export)
        #[clap(subcommand)]
        command: Option<ThemeCommand>,

        /// Theme name to operate on
        #[clap(long, short = 'n')]
        name: Option<String>,
    },
}

// Theme subcommands
#[derive(Subcommand, Clone)]
enum ThemeCommand {
    /// List available themes
    List,

    /// Create a new theme
    Create {
        /// Based on an existing theme
        #[clap(long)]
        base: Option<String>,

        /// Theme name
        #[clap(long, short = 'n')]
        name: String,

        /// Theme description
        #[clap(long, short = 'd')]
        description: Option<String>,
    },

    /// Set the active theme
    Set {
        /// Theme name
        #[clap(long, short = 'n', required = true)]
        name: String,
    },
}

// Input helper for rustyline
#[derive(Helper, Completer, Hinter, Validator, Highlighter)]
struct InputHelper;

// Main CLI handler
pub struct CLI {
    use_color: bool,
    recursive: bool,
    theme_manager: ThemeManager,
}

impl CLI {
    // Create a new CLI instance from args
    pub fn new(args: Args) -> Self {
        // Setup logger with appropriate level
        Self::setup_logger(args.log_level);

        // Check if color should be enabled
        let use_color = !args.no_color;

        // Initialize theme manager
        let mut theme_manager = ThemeManager::new();
        theme_manager.initialize();

        // Set the selected theme
        if !args.theme.is_empty() && args.theme != "default" {
            if let Err(e) = theme_manager.set_theme(&args.theme) {
                warn!("Could not set theme '{}': {}", args.theme, e);
                eprintln!("Warning: Could not set theme '{}': {}", args.theme, e);
            }
        }

        CLI {
            use_color,
            recursive: args.recursive,
            theme_manager,
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

        // Handle list_themes option
        if args.list_themes {
            return self.list_themes();
        }

        match args.command {
            Some(Command::Shell) => self.run_interactive_shell(),
            Some(Command::Theme { command, name }) => match command {
                Some(ThemeCommand::List) => self.list_themes(),
                Some(ThemeCommand::Create {
                    base,
                    name,
                    description,
                }) => self.create_theme(base, name, description),
                Some(ThemeCommand::Set { name }) => self.set_theme(&name),
                None => {
                    if let Some(theme_name) = name {
                        self.set_theme(&theme_name)
                    } else {
                        self.list_themes()
                    }
                }
            },
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

    // List available themes
    fn list_themes(&self) -> Result<(), String> {
        let themes = self.theme_manager.list_themes();
        let current_theme = self.theme_manager.current_theme();

        println!("Available themes:");
        for theme_name in themes {
            let is_current = theme_name == current_theme.name;
            if is_current && self.use_color {
                println!("  * {} (current)", theme_name.green());
            } else if is_current {
                println!("  * {} (current)", theme_name);
            } else {
                println!("    {}", theme_name);
            }
        }

        println!("\nUse --theme NAME to select a theme");
        Ok(())
    }

    // Set the current theme
    fn set_theme(&self, theme_name: &str) -> Result<(), String> {
        // Create a mutable clone for temporary modification
        let mut theme_manager = self.theme_manager.clone();

        match theme_manager.set_theme(theme_name) {
            Ok(_) => {
                println!("Theme set to: {}", theme_name);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // Create a new theme
    fn create_theme(
        &self,
        base: Option<String>,
        name: String,
        description: Option<String>,
    ) -> Result<(), String> {
        if name.is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }

        // Start with a base theme (default or specified)
        let mut theme_manager = self.theme_manager.clone();
        let mut theme = match base {
            Some(base_name) => {
                // Try to set theme to the base to get a copy
                match theme_manager.set_theme(&base_name) {
                    Ok(_) => theme_manager.current_theme().clone(),
                    Err(e) => return Err(format!("Base theme error: {}", e)),
                }
            }
            None => Theme::default(),
        };

        // Update the theme with the new name and description
        theme.name = name;
        if let Some(desc) = description {
            theme.description = desc;
        }

        // Save the new theme
        match theme_manager.create_theme(theme) {
            Ok(_) => {
                println!("Theme created successfully");
                Ok(())
            }
            Err(e) => Err(e),
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
            // Get the current theme
            let theme = self.theme_manager.current_theme();

            let prompt = if self.use_color {
                apply_color("lsql> ", &theme.colors.prompt, self.use_color).to_string()
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
                            let error_msg = display::format_message(
                                &format!("Error: {}", e),
                                "error",
                                theme,
                                self.use_color,
                            );
                            eprintln!("{}", error_msg);
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
                    let error_msg = display::format_message(
                        &format!("Error: {}", e),
                        "error",
                        theme,
                        self.use_color,
                    );
                    eprintln!("{}", error_msg);
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

        // Get the current theme
        let theme = self.theme_manager.current_theme();

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
            let message =
                display::format_message("No results found.", "warning", theme, self.use_color);
            println!("{}", message);
        } else {
            let table = display::display_entries(
                &filtered_entries,
                &query.selection,
                theme,
                self.use_color,
            );
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

            let message = display::format_message(&count_message, "success", theme, self.use_color);
            println!("{}", message);
        }

        Ok(())
    }
}
