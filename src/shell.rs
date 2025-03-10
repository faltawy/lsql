use crate::cli::CLI;
use log::info;
use nu_ansi_term::{Color, Style};
use reedline::{DefaultPrompt, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};
use std::borrow::Cow;

// Custom LSQL prompt
struct LSQLPrompt {
    base_prompt: DefaultPrompt,
}

impl LSQLPrompt {
    fn new() -> Self {
        // Create a simpler prompt using DefaultPrompt's constructor
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
    history_file: Option<String>,
}

impl LSQLShell {
    pub fn new() -> Self {
        // Set up history
        let history_file = dirs::home_dir()
            .map(|dir| dir.join(".lsql_history"))
            .map(|path| path.to_string_lossy().to_string());

        // Configure a simple line editor without highlighting or completion
        let line_editor = Reedline::create();

        Self {
            reedline: line_editor,
            history_file,
        }
    }

    pub fn print_welcome_message(&self) {
        println!(
            "{}",
            nu_ansi_term::Color::Cyan.paint(
                r#"
    ██╗      ███████╗ ██████╗ ██╗     
    ██║      ██╔════╝██╔═══██╗██║     
    ██║      ███████╗██║   ██║██║     
    ██║      ╚════██║██║▄▄ ██║██║     
    ███████╗ ███████║╚██████╔╝███████╗
    ╚══════╝ ╚══════╝ ╚══▀▀═╝ ╚══════╝
        "#
            )
        );
        println!(
            "{}",
            nu_ansi_term::Color::White
                .italic()
                .paint("SQL for your filesystem")
        );
        println!();
        println!(
            "Type {} to see available commands",
            nu_ansi_term::Color::Cyan.paint("help")
        );
        println!("Type {} to exit", nu_ansi_term::Color::Red.paint("exit"));
        println!();
    }

    fn process_command(&self, line: &str, cli: &CLI) -> bool {
        let command = line.trim().to_lowercase();

        match command.as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                return false;
            }
            "help" => {
                self.print_help();
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // ANSI escape code to clear screen
            }
            _ => {
                // Handle SQL query
                if command.starts_with("select") {
                    // Use the CLI's execute_query method
                    if let Err(e) = cli.execute_query(&command) {
                        println!("{}: {}", nu_ansi_term::Color::Red.bold().paint("Error"), e);
                    }
                } else if !command.is_empty() {
                    println!(
                        "{}: Unknown command '{}'",
                        nu_ansi_term::Color::Red.bold().paint("Error"),
                        command
                    );
                    println!(
                        "Type {} to see available commands",
                        nu_ansi_term::Color::Cyan.paint("help")
                    );
                }
            }
        }

        true
    }

    fn print_help(&self) {
        println!(
            "{}:",
            nu_ansi_term::Color::Yellow.bold().paint("LSQL Commands")
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
            "{}:",
            nu_ansi_term::Color::Yellow.bold().paint("Query Examples")
        );
        println!(
            "  {} - List all files and directories",
            nu_ansi_term::Color::Green.paint("SELECT * FROM .")
        );
        println!(
            "  {} - List only files",
            nu_ansi_term::Color::Green.paint("SELECT files FROM .")
        );
        println!(
            "  {} - List only directories",
            nu_ansi_term::Color::Green.paint("SELECT directories FROM .")
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
            "{}:",
            nu_ansi_term::Color::Yellow.bold().paint("Available Fields")
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
            let sig = self.reedline.read_line(prompt.as_ref());
            match sig {
                Ok(Signal::Success(line)) => {
                    // Process the command
                    if !self.process_command(&line, cli) {
                        break;
                    }
                }
                Ok(Signal::CtrlC) => {
                    println!("Ctrl-C received, press Ctrl-D or type 'exit' to exit");
                }
                Ok(Signal::CtrlD) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }
    }
}
