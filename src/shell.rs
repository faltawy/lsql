use crate::cli::CLI;
use log::{error, info, warn};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, DefaultPrompt, DefaultValidator, Emacs, Prompt, PromptEditMode,
    PromptHistorySearch, Reedline, Signal,
};

use std::borrow::Cow;
use std::io::{self, Write};

// Custom LSQL prompt
struct LSQLPrompt {
    base_prompt: DefaultPrompt,
}

impl LSQLPrompt {
    fn new() -> Self {
        Self {
            base_prompt: DefaultPrompt::default(),
        }
    }
}

impl Prompt for LSQLPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(format!(
            "{}> ",
            Style::new().fg(Color::Green).bold().paint("lsql")
        ))
    }

    fn render_prompt_right(&self) -> Cow<str> {
        self.base_prompt.render_prompt_right()
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        self.base_prompt.render_prompt_indicator(edit_mode)
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        self.base_prompt.render_prompt_multiline_indicator()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        self.base_prompt
            .render_prompt_history_search_indicator(history_search)
    }

    fn right_prompt_on_last_line(&self) -> bool {
        self.base_prompt.right_prompt_on_last_line()
    }
}

// The LSQL Shell
pub struct LSQLShell {
    reedline: Reedline,
}

enum ShellError {
    ExecutionError(String),
    UnknownCommand(String),
    IoError(io::Error),
}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::IoError(err)
    }
}

impl From<String> for ShellError {
    fn from(err: String) -> Self {
        ShellError::ExecutionError(err)
    }
}

impl LSQLShell {
    pub fn new() -> Self {
        // Configure a simple line editor without highlighting or completion
        let mut _keybindings = default_emacs_keybindings();
        let edit_mode = Box::new(Emacs::new(_keybindings));
        let line_editor = Reedline::create()
            .with_edit_mode(edit_mode)
            .with_validator(Box::new(DefaultValidator));
        Self {
            reedline: line_editor,
        }
    }

    pub fn print_welcome_message(&self) {
        println!(
            "{}",
            nu_ansi_term::Color::White
                .italic()
                .paint("SQL for your filesystem")
        );
        println!();
        println!(
            "{} Type {} to see available commands",
            Color::Cyan.bold().paint("•"),
            Color::Cyan.paint("help")
        );
        println!(
            "{} Type {} to exit",
            Color::Cyan.bold().paint("•"),
            Color::Red.paint("exit")
        );
        println!();
    }

    fn process_command(&self, line: &str, cli: &CLI) -> Result<bool, ShellError> {
        let command = line.trim().to_lowercase();

        // Empty command - just ignore
        if command.is_empty() {
            return Ok(true);
        }

        match command.as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                return Ok(false);
            }
            "help" => {
                self.print_help();
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear screen
                io::stdout().flush()?;
            }
            _ => match cli.execute_query(&command) {
                Ok(_) => (),
                Err(e) => return Err(ShellError::ExecutionError(e)),
            },
        }

        Ok(true)
    }

    fn display_error(&self, error: ShellError) {
        match error {
            ShellError::ExecutionError(msg) => {
                eprintln!("{} {}", Color::Red.bold().paint("Error:"), msg);
                error!("Query execution error: {}", msg);
            }
            ShellError::UnknownCommand(cmd) => {
                eprintln!(
                    "{} Unknown command '{}'",
                    Color::Red.bold().paint("Error:"),
                    cmd
                );
                eprintln!(
                    "    {} Type {} to see available commands",
                    Color::Cyan.bold().paint("•"),
                    Color::Cyan.paint("help")
                );
                warn!("Unknown command attempted: {}", cmd);
            }
            ShellError::IoError(err) => {
                eprintln!("{} I/O error: {}", Color::Red.bold().paint("Error:"), err);
                error!("I/O error: {}", err);
            }
        }
    }

    fn print_help(&self) {
        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                .bold()
                .underline()
                .paint("LSQL Commands")
        );
        println!(
            "  {} - Show this help message",
            nu_ansi_term::Color::Cyan.paint("help")
        );
        println!(
            "  {} - Clear the screen",
            nu_ansi_term::Color::Cyan.paint("clear")
        );
        println!(
            "  {} - Exit the shell",
            nu_ansi_term::Color::Cyan.paint("exit")
        );
        println!();

        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                .bold()
                .underline()
                .paint("Query Examples")
        );
        println!(
            "  {} - List all files and directories",
            nu_ansi_term::Color::Green.paint("SELECT * FROM .")
        );
        println!(
            "  {} - List file type information",
            nu_ansi_term::Color::Green.paint("SELECT type FROM .")
        );
        println!(
            "  {} - List name and type information",
            nu_ansi_term::Color::Green.paint("SELECT name, type FROM .")
        );
        println!(
            "  {} - List files with size > 1MB",
            nu_ansi_term::Color::Green.paint("SELECT * FROM . WHERE size > 1mb")
        );
        println!(
            "  {} - List files with specific extension",
            nu_ansi_term::Color::Green.paint("SELECT * FROM . WHERE ext = \"rs\"")
        );
        println!();

        println!(
            "{}",
            nu_ansi_term::Color::Yellow
                .bold()
                .underline()
                .paint("Available Fields")
        );
        println!(
            "  {} - Name of file or directory",
            nu_ansi_term::Color::Cyan.paint("name")
        );
        println!("  {} - Full path", nu_ansi_term::Color::Cyan.paint("path"));
        println!("  {} - File size", nu_ansi_term::Color::Cyan.paint("size"));
        println!(
            "  {} - Last modified time",
            nu_ansi_term::Color::Cyan.paint("modified")
        );
        println!(
            "  {} - Creation time",
            nu_ansi_term::Color::Cyan.paint("created")
        );
        println!(
            "  {} - File extension",
            nu_ansi_term::Color::Cyan.paint("ext")
        );
        println!(
            "  {} - File permissions",
            nu_ansi_term::Color::Cyan.paint("permissions")
        );
        println!(
            "  {} - Owner of file",
            nu_ansi_term::Color::Cyan.paint("owner")
        );
        println!(
            "  {} - Whether file is hidden",
            nu_ansi_term::Color::Cyan.paint("is_hidden")
        );
        println!(
            "  {} - Whether file is read-only",
            nu_ansi_term::Color::Cyan.paint("is_readonly")
        );
    }

    pub fn run(&mut self, cli: &CLI) {
        info!("Starting LSQL interactive shell");
        self.print_welcome_message();

        let prompt = Box::new(LSQLPrompt::new());

        loop {
            match self.reedline.read_line(prompt.as_ref()) {
                Ok(Signal::Success(line)) => {
                    // Process the command
                    match self.process_command(&line, cli) {
                        Ok(continue_loop) => {
                            if !continue_loop {
                                break;
                            }
                        }
                        Err(err) => {
                            self.display_error(err);
                        }
                    }
                }
                Ok(Signal::CtrlC) => {
                    println!(
                        "{} Press Ctrl+D or type 'exit' to exit",
                        Color::Yellow.bold().paint("Interrupted!")
                    );
                }
                Ok(Signal::CtrlD) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("{} {}", Color::Red.bold().paint("Terminal error:"), err);
                    error!("Terminal error: {}", err);
                    break;
                }
            }
        }

        info!("LSQL interactive shell exited");
    }
}
