// lsql - A simple SQL-like language interpreter to query the files
// like ls but supercharged with SQL-like queries
pub mod files;
pub mod parser;
use std::{error::Error, fs, io::Write, path::{Path, PathBuf}};
use chrono::{DateTime, Utc};
use files::{FileInfo, FileType};
use parser::parse;
use walkdir::WalkDir;
use colored::Colorize;


fn list_dir_contents(path: &Path) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Other
        };
        let last_modified = DateTime::<Utc>::from(metadata.modified()?);
        let file_info = FileInfo {
            size: metadata.len(),
            modified: last_modified,
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().display().to_string(),
            file_type,
        };
        files.push(file_info);
    }
    Ok(files)
}
struct State {
    files: Vec<FileInfo>,
    path: PathBuf,
}

impl State {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let current_dir = std::env::current_dir()?;
        let files = list_dir_contents(&current_dir)?;
        Ok(State {
            files,
            path: current_dir,
        })
    }

    pub fn set_path(&self, path: &Path) -> Result<Self, Box<dyn Error>> {
        let abs_path = fs::canonicalize(path)?;
        let files = list_dir_contents(&abs_path)?;
        Ok(State {
            files,
            path: abs_path,
        })
    }


   pub fn cd_back(&mut self) -> Result<Self, Box<dyn Error>>{
    let parent_path = self.path.parent().ok_or("No parent directory")?;
    self.set_path(parent_path)
    }

    pub fn get_abs_path(&self) -> String {
        self.path.display().to_string()
    }

}

fn main() -> ! {
    
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    let mut state = State::new().expect("Failed to initialize state");
    let args: Vec<String> = std::env::args().skip(1).collect();


    loop {
        let lsql_prompt = "lsql> ".green();
        println!("current directory: {}", state.get_abs_path());
        print!("{} ", lsql_prompt);
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        match parse(input) {
            Ok((_remaining, commands)) => {
                if let Some(first_command) = commands.first() {
                    match first_command {
                        parser::Command::Show => {
                            let query_set = files::FileQuerySet::new(state.files.clone());
                            let table = query_set.table_them();
                            println!("{}", table);
                        }
                        parser::Command::ChangeDir { path } => {
                            let result = if path == ".." {
                                state.cd_back()
                            } else {
                                state.set_path(Path::new(path))
                            };

                            match result {
                                Ok(new_state) => {
                                    state = new_state;
                                    // Reflect the change immediately
                                    let current_abs_path = state.get_abs_path();
                                },
                                Err(e) => eprintln!("Error: {}", e),
                            }
                        }
                        _ => {
                            println!("Command not implemented yet");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
