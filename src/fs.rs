// File System Module
// This module handles all file system operations like listing directories and retrieving file information

use chrono::{DateTime, Local};
use log::{debug, warn};
use std::fs::{self, Metadata};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use walkdir::{DirEntry, WalkDir};

use crate::parser::{ConditionNode, SelectionType};

// Represents a file or directory entry with its attributes
#[derive(Debug, Clone)]
pub struct FSEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_hidden: bool,
    pub modified: DateTime<Local>,
    pub created: DateTime<Local>,
    pub extension: Option<String>,
    pub permissions: String, // Simplified permissions string
}

impl FSEntry {
    // Create a new FSEntry from a DirEntry
    pub fn from_dir_entry(entry: DirEntry) -> Result<Self, String> {
        let path = entry.path();
        debug!("Processing entry: {}", path.display());

        let metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(e) => {
                warn!("Failed to read metadata for {}: {}", path.display(), e);
                return Err(format!("Failed to read metadata: {}", e));
            }
        };

        // Get file name
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                warn!("Failed to get file name for {}", path.display());
                return Err("Failed to get file name".to_string());
            }
        };

        // Get file extension
        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        // Get file times
        let modified = match get_datetime_from_metadata(&metadata, MetadataTime::Modified) {
            Ok(time) => time,
            Err(e) => {
                warn!("Failed to get modified time: {}", e);
                Local::now() // Default to current time
            }
        };

        let created = match get_datetime_from_metadata(&metadata, MetadataTime::Created) {
            Ok(time) => time,
            Err(e) => {
                warn!("Failed to get creation time: {}", e);
                Local::now() // Default to current time
            }
        };

        // Check if file is hidden (starts with . on Unix, or has hidden attribute on Windows)
        let is_hidden = name.starts_with('.');

        // Get permissions
        let permissions = format_permissions(&metadata);

        Ok(FSEntry {
            name,
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_hidden,
            modified,
            created,
            extension,
            permissions,
        })
    }
}

// Types of time metadata
enum MetadataTime {
    Modified,
    Created,
}

// Convert system time to DateTime
fn get_datetime_from_metadata(
    metadata: &Metadata,
    time_type: MetadataTime,
) -> Result<DateTime<Local>, String> {
    let system_time = match time_type {
        MetadataTime::Modified => metadata
            .modified()
            .map_err(|e| format!("Failed to get modified time: {}", e))?,
        MetadataTime::Created => metadata
            .created()
            .map_err(|e| format!("Failed to get creation time: {}", e))?,
    };

    let duration = match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(e) => {
            return Err(format!(
                "Time appears to be before Unix epoch: {}",
                e.duration().as_secs()
            ))
        }
    };

    let secs = duration.as_secs() as i64;
    let nsecs = duration.subsec_nanos() as u32;

    // Convert to DateTime<Local>
    match DateTime::from_timestamp(secs, nsecs) {
        Some(dt) => Ok(DateTime::<Local>::from(dt)),
        None => Ok(Local::now()),
    }
}

// Format permissions in a simplified way
fn format_permissions(metadata: &Metadata) -> String {
    let permissions = metadata.permissions();
    if permissions.readonly() {
        "readonly".to_string()
    } else {
        "readwrite".to_string()
    }
}

// List entries in a directory with optional filtering
pub fn list_entries(
    path: &str,
    selection: &SelectionType,
    condition: &Option<ConditionNode>,
    limit: Option<u64>,
    recursive: bool,
) -> Result<Vec<FSEntry>, String> {
    let path = normalize_path(path)?;
    debug!("Listing entries in: {}", path.display());

    let mut entries = Vec::new();
    let walker = if recursive {
        WalkDir::new(path).into_iter()
    } else {
        WalkDir::new(path).max_depth(1).into_iter()
    };

    for result in walker {
        match result {
            Ok(entry) => {
                // Skip the root directory itself when non-recursive
                if !recursive && entry.depth() == 0 {
                    continue;
                }

                match FSEntry::from_dir_entry(entry) {
                    Ok(fs_entry) => {
                        // Filter based on selection type
                        let include = match selection {
                            SelectionType::All => true,
                            SelectionType::Files => fs_entry.is_file,
                            SelectionType::Directories => fs_entry.is_dir,
                            SelectionType::Fields(_) => true, // Fields selection doesn't affect filtering
                        };

                        if include {
                            // Add entry if it passes the selection filter
                            entries.push(fs_entry);
                        }
                    }
                    Err(e) => warn!("Error creating FSEntry: {}", e),
                }
            }
            Err(e) => warn!("Error walking directory: {}", e),
        }
    }

    // Apply condition filtering using the filter module
    let mut filtered_entries = crate::filter::filter_entries(entries, condition);

    // Apply limit if specified
    if let Some(limit_val) = limit {
        if filtered_entries.len() > limit_val as usize {
            debug!("Limiting results to {} entries", limit_val);
            filtered_entries.truncate(limit_val as usize);
        }
    }

    debug!("Found {} entries after filtering", filtered_entries.len());
    Ok(filtered_entries)
}

// Normalize a path string to a PathBuf
fn normalize_path(path_str: &str) -> Result<PathBuf, String> {
    let path_str = path_str.trim();

    // Handle special case for current directory
    if path_str == "." {
        return Ok(std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?);
    }

    // Handle home directory expansion
    if path_str.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy();
            let expanded = path_str.replacen('~', &home_str, 1);
            return Ok(PathBuf::from(expanded));
        }
    }

    Ok(PathBuf::from(path_str))
}

// Context for search operations
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub recursive: bool,
}

impl SearchContext {
    pub fn new(recursive: bool) -> Self {
        SearchContext { recursive }
    }
}

// Execute a query and return matching entries
pub fn execute_query(
    query: &crate::parser::Query,
    path: &str,
    context: &SearchContext,
) -> Result<Vec<FSEntry>, String> {
    let path_to_search = if query.path.is_empty() {
        path
    } else {
        &query.path
    };

    list_entries(
        path_to_search,
        &query.selection,
        &query.condition,
        query.limit,
        context.recursive,
    )
}
