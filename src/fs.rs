// File System Module
// This module handles all file system operations like listing directories and retrieving file information

use chrono::{DateTime, Local};
use log::{debug, warn};
use std::fs::{self, Metadata};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use walkdir::{DirEntry, WalkDir};

use crate::parser::{ComparisonOperator, ConditionNode, LogicalOperator, SelectionType, Value};

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

        // Check if it's hidden (starts with .)
        let is_hidden = name.starts_with('.');

        // Get file extension
        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        // Get timestamps
        let modified = match get_datetime_from_metadata(&metadata, MetadataTime::Modified) {
            Ok(time) => time,
            Err(e) => {
                warn!("Failed to get modified time for {}: {}", path.display(), e);
                return Err(e);
            }
        };

        let created = match get_datetime_from_metadata(&metadata, MetadataTime::Created) {
            Ok(time) => time,
            Err(e) => {
                warn!("Failed to get creation time for {}: {}", path.display(), e);
                return Err(e);
            }
        };

        // Get permissions
        let permissions = format_permissions(&metadata);

        debug!(
            "Created FSEntry for {}: is_dir={}, size={}",
            name,
            metadata.is_dir(),
            metadata.len()
        );

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

    // Check if the entry matches a condition
    pub fn matches_condition(&self, condition: &Option<ConditionNode>) -> bool {
        match condition {
            None => true,
            Some(node) => {
                let result = self.evaluate_condition_node(node);
                debug!("Condition evaluation for {}: {}", self.name, result);
                result
            }
        }
    }

    // Evaluate a condition node against this entry
    fn evaluate_condition_node(&self, node: &ConditionNode) -> bool {
        match node {
            ConditionNode::Leaf(condition) => self.evaluate_single_condition(
                &condition.identifier,
                &condition.operator,
                &condition.value,
            ),
            ConditionNode::Branch {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate_condition_node(left);
                let right_result = self.evaluate_condition_node(right);

                match operator {
                    LogicalOperator::And => left_result && right_result,
                    LogicalOperator::Or => left_result || right_result,
                }
            }
        }
    }

    // Evaluate a single condition against this entry
    fn evaluate_single_condition(
        &self,
        identifier: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool {
        debug!(
            "Evaluating condition: {} {:?} {:?}",
            identifier, operator, value
        );

        match identifier {
            "name" => self.compare_string_field(&self.name, operator, value),
            "path" => self.compare_string_field(&self.path, operator, value),
            "ext" => {
                if let Some(ext) = &self.extension {
                    self.compare_string_field(ext, operator, value)
                } else {
                    false
                }
            }
            "size" => self.compare_size_field(self.size, operator, value),
            "is_hidden" => {
                if let Value::Bool(b) = value {
                    match operator {
                        ComparisonOperator::Equal => self.is_hidden == *b,
                        ComparisonOperator::NotEqual => self.is_hidden != *b,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            "is_dir" => {
                if let Value::Bool(b) = value {
                    match operator {
                        ComparisonOperator::Equal => self.is_dir == *b,
                        ComparisonOperator::NotEqual => self.is_dir != *b,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            "is_file" => {
                if let Value::Bool(b) = value {
                    match operator {
                        ComparisonOperator::Equal => self.is_file == *b,
                        ComparisonOperator::NotEqual => self.is_file != *b,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            "permissions" => self.compare_string_field(&self.permissions, operator, value),
            // Timestamps would need more sophisticated comparison
            _ => {
                warn!("Unknown identifier in condition: {}", identifier);
                false
            }
        }
    }

    // Compare a string field with a value
    fn compare_string_field(
        &self,
        field: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool {
        if let Value::String(s) = value {
            match operator {
                ComparisonOperator::Equal => field == s,
                ComparisonOperator::NotEqual => field != s,
                ComparisonOperator::Like => field.contains(s),
                ComparisonOperator::Contains => field.contains(s),
                _ => false,
            }
        } else {
            false
        }
    }

    // Compare a size field with a value
    fn compare_size_field(&self, size: u64, operator: &ComparisonOperator, value: &Value) -> bool {
        match value {
            Value::Number(n) => {
                let size_f64 = size as f64;
                match operator {
                    ComparisonOperator::Equal => (size_f64 - n).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (size_f64 - n).abs() >= f64::EPSILON,
                    ComparisonOperator::LessThan => size_f64 < *n,
                    ComparisonOperator::LessOrEqual => size_f64 <= *n,
                    ComparisonOperator::GreaterThan => size_f64 > *n,
                    ComparisonOperator::GreaterOrEqual => size_f64 >= *n,
                    _ => false,
                }
            }
            Value::SizedNumber(n, unit) => {
                let bytes = convert_to_bytes(*n, unit);
                let size_f64 = size as f64;

                match operator {
                    ComparisonOperator::Equal => (size_f64 - bytes).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (size_f64 - bytes).abs() >= f64::EPSILON,
                    ComparisonOperator::LessThan => size_f64 < bytes,
                    ComparisonOperator::LessOrEqual => size_f64 <= bytes,
                    ComparisonOperator::GreaterThan => size_f64 > bytes,
                    ComparisonOperator::GreaterOrEqual => size_f64 >= bytes,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

// Enum to specify which timestamp to retrieve
enum MetadataTime {
    Modified,
    Created,
}

// Convert system time to DateTime<Local>
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

    let duration = system_time
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?;

    let secs = duration.as_secs();
    let nsecs = duration.subsec_nanos();

    // Convert to DateTime<Local>
    let datetime = DateTime::from_timestamp(secs as i64, nsecs)
        .ok_or_else(|| "Invalid timestamp".to_string())?
        .into();

    Ok(datetime)
}

// Format permissions as a simple string (read-only for now)
fn format_permissions(metadata: &Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        format!("{:o}", mode & 0o777)
    }

    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "readonly".to_string()
        } else {
            "readwrite".to_string()
        }
    }
}

// Convert a sized number (e.g., 10mb) to bytes
fn convert_to_bytes(num: f64, unit: &str) -> f64 {
    match unit.to_lowercase().as_str() {
        "kb" => num * 1024.0,
        "mb" => num * 1024.0 * 1024.0,
        "gb" => num * 1024.0 * 1024.0 * 1024.0,
        "tb" => num * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => num, // Assume bytes for any other unit
    }
}

// List entries in a directory
pub fn list_entries(
    path: &str,
    selection: &SelectionType,
    condition: &Option<ConditionNode>,
    recursive: bool,
) -> Result<Vec<FSEntry>, String> {
    debug!("Listing entries in {} (recursive={})", path, recursive);

    let path = match normalize_path(path) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to normalize path {}: {}", path, e);
            return Err(e);
        }
    };

    let mut walker = WalkDir::new(path);

    // Only go one level deep if not recursive
    if !recursive {
        walker = walker.max_depth(1);
    }

    let mut entries = vec![];

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() && entry.depth() == 0 {
            // Skip the root directory itself
            debug!("Skipping root directory itself");
            continue;
        }

        match FSEntry::from_dir_entry(entry) {
            Ok(fs_entry) => {
                // Apply selection filter
                let include = match selection {
                    SelectionType::All => true,
                    SelectionType::Files => fs_entry.is_file,
                    SelectionType::Directories => fs_entry.is_dir,
                    SelectionType::Fields(_) => true, // We'll handle field selection later
                };

                // Apply condition filter
                if include && fs_entry.matches_condition(condition) {
                    debug!("Adding entry to results: {}", fs_entry.name);
                    entries.push(fs_entry);
                }
            }
            Err(e) => {
                warn!("Error processing entry: {}", e);
            }
        }
    }

    debug!("Found {} entries after filtering", entries.len());

    Ok(entries)
}

// Normalize a path string
fn normalize_path(path_str: &str) -> Result<PathBuf, String> {
    let path = if path_str == "." {
        match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to get current directory: {}", e);
                return Err(format!("Failed to get current directory: {}", e));
            }
        }
    } else {
        PathBuf::from(path_str)
    };

    if !path.exists() {
        warn!("Path does not exist: {}", path.display());
        return Err(format!("Path does not exist: {}", path.display()));
    }

    debug!("Normalized path: {}", path.display());

    Ok(path)
}
