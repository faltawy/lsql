// CLI Module
// This module handles command-line arguments, interactive shell, and user interface

use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use log::{debug, info, warn};

use crate::display;
use crate::fs;
use crate::parser::LSQLParser;
use crate::theme::{Theme, ThemeManager};

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
    pub query: Option<String>,

    /// Disable colored output
    #[clap(long, short = 'n')]
    pub no_color: bool,

    /// Enable recursive search (default is non-recursive)
    #[clap(long, short = 'r')]
    pub recursive: bool,

    /// Set the logging level
    #[clap(long, short = 'l', value_enum, default_value = "off")]
    pub log_level: LogLevel,

    /// Select the color theme
    #[clap(long, short = 't', default_value = "default")]
    pub theme: String,

    /// List available themes
    #[clap(long)]
    pub list_themes: bool,

    /// Start interactive mode
    #[clap(long, short = 'i')]
    pub interactive: bool,

    /// Subcommands
    #[clap(subcommand)]
    pub command: Option<Command>,
}

// Define subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum Command {
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
#[derive(Subcommand, Clone, Debug)]
pub enum ThemeCommand {
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
            "Running with recursive: {}, color: {}",
            args.recursive, !args.no_color
        );

        if args.list_themes {
            return self.list_themes();
        }

        // Handle subcommands
        if let Some(command) = args.command {
            match command {
                Command::Shell => {
                    // Handled in main.rs now
                    return Ok(());
                }
                Command::Theme { command, name } => {
                    debug!("Theme command: {:?}, name: {:?}", command, name);

                    // Handle theme command
                    match command {
                        Some(ThemeCommand::List) => {
                            return self.list_themes();
                        }
                        Some(ThemeCommand::Create {
                            base,
                            name,
                            description,
                        }) => {
                            debug!(
                                "Creating theme '{}' based on '{:?}' with description '{:?}'",
                                name, base, description
                            );
                            return self.create_theme(base, name, description);
                        }
                        Some(ThemeCommand::Set { name }) => {
                            debug!("Setting theme '{}'", name);
                            return self.set_theme(&name);
                        }
                        None => {
                            // If no subcommand is provided but a name is provided, assume "set"
                            if let Some(name) = name {
                                debug!("Setting theme '{}'", name);
                                return self.set_theme(&name);
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }

        // If interactive flag is set, we'll handle it in main.rs
        if args.interactive {
            return Ok(());
        }

        // Execute query if provided
        if let Some(query) = args.query {
            return self.execute_query(&query);
        }

        // Neither query nor subcommand provided
        eprintln!("No query or subcommand provided. Use --help for usage information.");
        Ok(())
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

    // Execute a single query
    pub fn execute_query(&self, query_str: &str) -> Result<(), String> {
        debug!("Executing query: {}", query_str);

        // Parse query
        let query = LSQLParser::parse_query(query_str)?;
        debug!("Parsed query: {:?}", query);

        // Build search context
        let path = query.path.clone();
        let search_context = fs::SearchContext::new(self.recursive);

        // Execute query to get the results
        let results = fs::execute_query(&query, &path, &search_context)?;
        debug!("Found {} items matching query", results.len());

        // Get the current theme
        let theme = self.theme_manager.current_theme();

        // Display results
        display::display_results(&results, &query.selection, theme, self.use_color)?;

        // Print count
        if !results.is_empty() {
            info!("{} items found", results.len());
            let count_message = format!("{} items found", results.len());
            let message = display::format_message(&count_message, "success", theme, self.use_color);
            println!("{}", message);
        }

        Ok(())
    }
}
