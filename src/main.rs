// lsql - A simple SQL-like language interpreter to query the files
// like ls but supercharged with SQL-like queries
pub mod files;
pub mod parser;

use std::result;
#[allow(unused, unused_variables, dead_code)]
use std::{error::Error, fs, io::Write};

use chrono::{DateTime, Utc};
use files::{FileInfo, FileType};
use parser::parse;
use walkdir::WalkDir;

fn list_dir_contents(path: &str) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Other
        };
        let last_modified = || {
            let time: DateTime<Utc> = metadata.modified().unwrap().into();
            time
        };
        let file_info = FileInfo {
            size: metadata.len(),
            modified: last_modified(),
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
    path: String,
}

impl State {
    pub fn new() -> Self {
        State {
            files: list_dir_contents(".").unwrap(),
            path: String::from("."),
        }
    }

    pub fn set_path(&mut self, path: &str) {
        match fs::canonicalize(path) {
            Ok(abs_path) => {
                self.path = abs_path.to_str().unwrap().to_string();
                self.files = list_dir_contents(&self.path).unwrap();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    pub fn get_abs_path(&self) -> String {
        let abs_path = fs::canonicalize(&self.path).unwrap();
        abs_path.display().to_string()
    }
}

fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }
    let mut state = State::new();
    let args: Vec<String> = std::env::args().skip(1).collect();

    loop {
        println!("lsql/path> {}", &state.path);
        print!("lsql> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let parsed = parse(input);
        println!("{:?}", parsed);


        match parsed {
            Ok(result) => {
                
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
