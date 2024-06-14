use chrono::{self, DateTime, Utc};
use lsql_parser::{self, LSQLCommand, Parser};
#[allow(unused, unused_variables, dead_code)]
// lsql - A simple SQL-like language interpreter to query the files
// like ls but supercharged with SQL-like queries
use std::{error::Error, fs, io::Write};
use walkdir::WalkDir;
#[derive(Debug)]
enum FileType {
    Directory,
    File,
    Other,
}

#[derive(Debug)]
enum FilePermission {
    Read,
    Write,
    Execute,
}

#[derive(Debug)]
struct FileInfo {
    size: u64,
    modified: chrono::DateTime<Utc>,
    name: String,
    file_type: FileType,
    path: String,
}
impl FileInfo {
    pub fn human_readable_size(&self) -> String {
        let size = self.size;
        let kb = 1024;
        let mb = kb * 1024;
        let gb = mb * 1024;
        let tb = gb * 1024;
        if size < kb {
            format!("{} B", size)
        } else if size < mb {
            format!("{:.2} KB", size as f64 / kb as f64)
        } else if size < gb {
            format!("{:.2} MB", size as f64 / mb as f64)
        } else if size < tb {
            format!("{:.2} GB", size as f64 / gb as f64)
        } else {
            format!("{:.2} TB", size as f64 / tb as f64)
        }
    }
}

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
fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    let args = std::env::args().skip(1);
    let binding = fs::canonicalize(args.last().unwrap_or(String::from("."))).unwrap();
    let abs_path = binding.to_str().unwrap();

    loop {
        println!("lsql/path> {}", abs_path);
        print!("lsql> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        let parsed = Parser::from_tokens_str(input).walk();
        let first = parsed.first();

        match first {
            None {} => {
                println!("Invalid command");
            }
            Some(first) => match first {
                LSQLCommand::Cd { to } => {
                    println!("Changing directory to {}", to);
                }
                _ => {
                    println!("Invalid command");
                }
            },
        }
    }
}
