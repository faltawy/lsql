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

    // Get the entries
    let mut entries = list_entries(
        path_to_search,
        &query.selection,
        &query.condition,
        query.limit,
        context.recursive,
    )?;

    // Apply sorting if ORDER BY is specified
    if !query.order_by.is_empty() {
        sort_entries(&mut entries, &query.order_by);
    }

    Ok(entries)
}

// Sort entries based on ORDER BY terms
fn sort_entries(entries: &mut Vec<FSEntry>, order_by: &[crate::parser::OrderTerm]) {
    use crate::parser::OrderDirection;
    use std::cmp::Ordering;

    entries.sort_by(|a, b| {
        // Apply each ORDER BY term in sequence
        for term in order_by {
            let ordering = match term.field.as_str() {
                "name" => compare_strings(&a.name, &b.name),
                "path" => compare_strings(&a.path, &b.path),
                "size" => compare_numbers(a.size, b.size),
                "modified" => compare_dates(&a.modified, &b.modified),
                "created" => compare_dates(&a.created, &b.created),
                "ext" => compare_options(&a.extension, &b.extension),
                "permissions" => compare_strings(&a.permissions, &b.permissions),
                "is_hidden" => compare_bools(a.is_hidden, b.is_hidden),
                "is_readonly" => compare_strings(&a.permissions, &b.permissions),
                _ => Ordering::Equal, // Unknown field, no sorting
            };

            // If this term gives a non-equal result, return it (possibly reversed for DESC)
            if ordering != Ordering::Equal {
                return if term.direction == OrderDirection::Ascending {
                    ordering
                } else {
                    ordering.reverse()
                };
            }
        }

        // If all terms are equal, maintain original order
        Ordering::Equal
    });
}

// Helper functions for comparing different types
fn compare_strings(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

fn compare_numbers(a: u64, b: u64) -> std::cmp::Ordering {
    a.cmp(&b)
}

fn compare_dates(
    a: &chrono::DateTime<chrono::Local>,
    b: &chrono::DateTime<chrono::Local>,
) -> std::cmp::Ordering {
    a.cmp(b)
}

fn compare_options<T: Ord>(a: &Option<T>, b: &Option<T>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a_val), Some(b_val)) => a_val.cmp(b_val),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn compare_bools(a: bool, b: bool) -> std::cmp::Ordering {
    a.cmp(&b)
}

// Delete entries that match the criteria
pub fn delete_entries(
    path: &str,
    selection: &SelectionType,
    condition: &Option<ConditionNode>,
    limit: Option<u64>,
    recursive: bool,
    dry_run: bool,
) -> Result<(Vec<FSEntry>, usize), String> {
    // First, list all entries that match the criteria
    let entries_to_delete = list_entries(path, selection, condition, limit, recursive)?;

    if entries_to_delete.is_empty() {
        return Ok((Vec::new(), 0));
    }

    debug!("Found {} entries to delete", entries_to_delete.len());

    // If this is a dry run, just return the entries that would be deleted
    if dry_run {
        return Ok((entries_to_delete, 0));
    }

    let mut deleted_count = 0;
    let mut failed_entries = Vec::new();

    // Delete each entry
    for entry in &entries_to_delete {
        let path = std::path::Path::new(&entry.path);

        let result = if entry.is_dir {
            if recursive {
                // If recursive flag is set, delete directory and all its contents
                debug!("Recursively deleting directory: {}", entry.path);
                std::fs::remove_dir_all(path)
            } else {
                // If not recursive, check if directory is empty first
                match std::fs::read_dir(path) {
                    Ok(mut dir_iter) => {
                        if dir_iter.next().is_none() {
                            // Directory is empty, safe to delete
                            debug!("Deleting empty directory: {}", entry.path);
                            std::fs::remove_dir(path)
                        } else {
                            // Directory not empty and recursive flag not set
                            debug!(
                                "Skipping non-empty directory (recursive flag not set): {}",
                                entry.path
                            );
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Directory not empty (use recursive flag to delete)",
                            ))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        } else {
            debug!("Deleting file: {}", entry.path);
            std::fs::remove_file(path)
        };

        match result {
            Ok(_) => {
                deleted_count += 1;
            }
            Err(e) => {
                warn!("Failed to delete {}: {}", entry.path, e);

                // Add more descriptive error message based on error kind
                let error_msg = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        format!("Permission denied: {}", entry.path)
                    }
                    std::io::ErrorKind::NotFound => {
                        format!("No such file or directory: {}", entry.path)
                    }
                    std::io::ErrorKind::DirectoryNotEmpty => format!(
                        "Directory not empty (use recursive flag to delete): {}",
                        entry.path
                    ),
                    _ => format!("Failed to delete {}: {}", entry.path, e),
                };

                let mut failed_entry = entry.clone();
                failed_entry.name = format!("{} - {}", failed_entry.name, error_msg);
                failed_entries.push(failed_entry);
            }
        }
    }

    debug!("Successfully deleted {} entries", deleted_count);

    Ok((failed_entries, deleted_count))
}

// Execute a delete query and return the results
pub fn execute_delete_query(
    query: &crate::parser::Query,
    path: &str,
    context: &SearchContext,
    dry_run: bool,
) -> Result<(Vec<FSEntry>, usize), String> {
    let path_to_search = if query.path.is_empty() {
        path
    } else {
        &query.path
    };

    // Use the is_recursive flag from the query if it's a DELETE query,
    // otherwise fall back to the context's recursive flag
    let recursive = if query.query_type == crate::parser::QueryType::Delete {
        query.is_recursive || context.recursive
    } else {
        context.recursive
    };

    delete_entries(
        path_to_search,
        &query.selection,
        &query.condition,
        query.limit,
        recursive,
        dry_run,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        ComparisonOperator, Condition, ConditionNode, OrderDirection, SelectionType, Value,
    };
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to create a temporary directory with test files
    fn setup_test_directory() -> tempfile::TempDir {
        let dir = tempdir().unwrap();

        // Create some test files
        for i in 1..10 {
            let file_path = dir.path().join(format!("test_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Test content {}", i).unwrap();
        }

        // Create a subdirectory
        let subdir_path = dir.path().join("subdir");
        std::fs::create_dir(&subdir_path).unwrap();

        // Create files in the subdirectory
        for i in 1..5 {
            let file_path = subdir_path.join(format!("subdir_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Subdir content {}", i).unwrap();
        }

        dir
    }

    #[test]
    fn test_limit_entries() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Test with limit of 3
        let entries = list_entries(&dir_path, &SelectionType::All, &None, Some(3), false).unwrap();

        assert_eq!(entries.len(), 3, "Should return exactly 3 entries");

        // Test with limit of 0 (should return empty list)
        let zero_entries =
            list_entries(&dir_path, &SelectionType::All, &None, Some(0), false).unwrap();

        assert_eq!(
            zero_entries.len(),
            0,
            "Should return empty list with limit 0"
        );

        // Test with limit larger than available entries
        let all_entries =
            list_entries(&dir_path, &SelectionType::All, &None, Some(100), false).unwrap();

        // Should return all entries in the directory (not recursive)
        assert!(all_entries.len() > 0, "Should return all entries");
        assert!(
            all_entries.len() <= 10,
            "Should not exceed actual entry count"
        );

        // Test with no limit
        let unlimited_entries =
            list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        assert_eq!(
            unlimited_entries.len(),
            all_entries.len(),
            "No limit should return all entries"
        );
    }

    #[test]
    fn test_limit_with_recursive() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Test with limit and recursive flag
        let entries = list_entries(&dir_path, &SelectionType::All, &None, Some(5), true).unwrap();

        assert_eq!(
            entries.len(),
            5,
            "Should return exactly 5 entries with recursive search"
        );

        // Get all entries recursively
        let all_entries = list_entries(&dir_path, &SelectionType::All, &None, None, true).unwrap();

        // Should include files from subdirectories
        assert!(
            all_entries.len() > 10,
            "Should include files from subdirectories"
        );
    }

    #[test]
    fn test_limit_with_selection_and_condition() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Test with files selection and limit
        let file_entries =
            list_entries(&dir_path, &SelectionType::All, &None, Some(4), false).unwrap();

        assert_eq!(
            file_entries.len(),
            4,
            "Should return exactly 4 file entries"
        );

        // Create a simple condition for testing
        let condition = Some(ConditionNode::Leaf(Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Contains,
            value: Value::String("test_file".to_string()),
        }));

        // Test with condition and limit
        let filtered_entries =
            list_entries(&dir_path, &SelectionType::All, &condition, Some(2), false).unwrap();

        assert!(filtered_entries.len() <= 2, "Should not exceed limit of 2");

        // Verify that the condition was applied before the limit
        for entry in &filtered_entries {
            assert!(
                entry.name.contains("test_file"),
                "Entry should match the condition"
            );
        }
    }

    #[test]
    fn test_delete_entries_dry_run() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Test with dry run mode (should not delete anything)
        let (entries_to_delete, deleted_count) = delete_entries(
            &dir_path,
            &SelectionType::All,
            &None,
            Some(3),
            false,
            true, // dry run
        )
        .unwrap();

        // Should return entries that would be deleted
        assert_eq!(
            entries_to_delete.len(),
            3,
            "Should return 3 entries that would be deleted"
        );

        // Should not delete anything
        assert_eq!(
            deleted_count, 0,
            "Should not delete any entries in dry run mode"
        );

        // Verify files still exist
        let all_entries = list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        assert!(
            all_entries.len() > 3,
            "Files should still exist after dry run"
        );
    }

    #[test]
    fn test_delete_entries_with_condition() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create a condition to match specific files
        let condition = Some(ConditionNode::Leaf(Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Contains,
            value: Value::String("test_file_1".to_string()), // Only match test_file_1.txt
        }));

        // Count files before deletion
        let before_entries =
            list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        let before_count = before_entries.len();

        // Delete entries matching the condition
        let (failed_entries, deleted_count) = delete_entries(
            &dir_path,
            &SelectionType::All,
            &condition,
            None,
            false,
            false, // actual deletion
        )
        .unwrap();

        // Should have deleted 1 file
        assert_eq!(deleted_count, 1, "Should delete 1 file");
        assert_eq!(
            failed_entries.len(),
            0,
            "Should not have any failed deletions"
        );

        // Verify file was deleted
        let after_entries =
            list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        assert_eq!(
            after_entries.len(),
            before_count - 1,
            "One file should be deleted"
        );

        // Verify the specific file was deleted
        let has_test_file_1 = after_entries.iter().any(|e| e.name.contains("test_file_1"));
        assert!(!has_test_file_1, "test_file_1.txt should be deleted");
    }

    #[test]
    fn test_delete_entries_recursive() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create a nested directory structure
        let nested_dir_path = temp_dir.path().join("nested_dir");
        std::fs::create_dir(&nested_dir_path).unwrap();

        // Create files in the nested directory
        for i in 1..3 {
            let file_path = nested_dir_path.join(format!("nested_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Nested content {}", i).unwrap();
        }

        // Count directories before deletion
        let before_dirs = list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        let before_dir_count = before_dirs.len();
        assert!(before_dir_count > 0, "Should have at least one directory");

        // Try to delete a directory without recursive flag (should fail)
        let condition = Some(ConditionNode::Leaf(Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("nested_dir".to_string()),
        }));

        let (failed_entries, deleted_count) = delete_entries(
            &dir_path,
            &SelectionType::All,
            &condition, // Add condition to target specific directory
            None,       // Remove limit since we're using condition
            false,      // non-recursive
            false,      // actual deletion
        )
        .unwrap();

        // Should have failed to delete the directory because it's not empty
        assert_eq!(
            deleted_count, 0,
            "Should not delete any directories without recursive flag"
        );

        // Now delete with recursive flag
        let (failed_entries, deleted_count) = delete_entries(
            &dir_path,
            &SelectionType::All,
            &condition, // Use same condition to target the specific directory
            None,
            true,  // recursive
            false, // actual deletion
        )
        .unwrap();

        // Should have successfully deleted the directory
        assert_eq!(deleted_count, 1, "Should delete exactly one directory");
        assert_eq!(
            failed_entries.len(),
            0,
            "Should not have any failed deletions"
        );

        // Verify directory was deleted
        let after_dirs = list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();
        assert!(
            !after_dirs.iter().any(|e| e.name == "nested_dir"),
            "nested_dir should be deleted"
        );
    }

    #[test]
    fn test_delete_entries_with_nested_structure() {
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create a deeper nested directory structure
        let level1_dir = temp_dir.path().join("level1");
        std::fs::create_dir(&level1_dir).unwrap();

        let level2_dir = level1_dir.join("level2");
        std::fs::create_dir(&level2_dir).unwrap();

        let level3_dir = level2_dir.join("level3");
        std::fs::create_dir(&level3_dir).unwrap();

        // Create files at each level
        for i in 1..3 {
            let file_path = level1_dir.join(format!("level1_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Level 1 content {}", i).unwrap();

            let file_path = level2_dir.join(format!("level2_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Level 2 content {}", i).unwrap();

            let file_path = level3_dir.join(format!("level3_file_{}.txt", i));
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "Level 3 content {}", i).unwrap();
        }

        // Count all entries recursively before deletion
        let before_entries = list_entries(
            &dir_path,
            &SelectionType::All,
            &None,
            None,
            true, // recursive
        )
        .unwrap();

        let before_count = before_entries.len();

        // Delete the top-level directory recursively
        let (failed_entries, deleted_count) = delete_entries(
            &dir_path,
            &SelectionType::All,
            &Some(ConditionNode::Leaf(Condition {
                identifier: "name".to_string(),
                operator: ComparisonOperator::Equal,
                value: Value::String("level1".to_string()),
            })),
            None,
            true,  // recursive
            false, // actual deletion
        )
        .unwrap();

        // Should have successfully deleted the directory and all its contents
        assert_eq!(deleted_count, 1, "Should delete one directory");
        assert_eq!(
            failed_entries.len(),
            0,
            "Should not have any failed deletions"
        );

        // Verify directory and all its contents were deleted
        let after_entries = list_entries(
            &dir_path,
            &SelectionType::All,
            &None,
            None,
            true, // recursive
        )
        .unwrap();

        // The count should be reduced by at least 9 entries (3 directories + 6 files)
        assert!(
            before_count - after_entries.len() >= 9,
            "Should have deleted at least 9 entries (3 dirs + 6 files)"
        );

        // Verify the level1 directory no longer exists
        let has_level1 = after_entries.iter().any(|e| e.name == "level1");
        assert!(!has_level1, "level1 directory should be deleted");
    }

    #[test]
    fn test_sort_entries() {
        // Create a test directory with files of different sizes and names
        let temp_dir = setup_test_directory();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Get all entries
        let mut entries = list_entries(&dir_path, &SelectionType::All, &None, None, false).unwrap();

        // Ensure we have at least 3 entries for testing
        assert!(
            entries.len() >= 3,
            "Need at least 3 entries for sorting test"
        );

        // Sort by name ascending
        let order_by_name_asc = vec![crate::parser::OrderTerm {
            field: "name".to_string(),
            direction: OrderDirection::Ascending,
        }];

        sort_entries(&mut entries, &order_by_name_asc);

        // Verify entries are sorted by name ascending
        for i in 1..entries.len() {
            assert!(
                entries[i - 1].name <= entries[i].name,
                "Entries should be sorted by name ascending"
            );
        }

        // Sort by name descending
        let order_by_name_desc = vec![crate::parser::OrderTerm {
            field: "name".to_string(),
            direction: OrderDirection::Descending,
        }];

        sort_entries(&mut entries, &order_by_name_desc);

        // Verify entries are sorted by name descending
        for i in 1..entries.len() {
            assert!(
                entries[i - 1].name >= entries[i].name,
                "Entries should be sorted by name descending"
            );
        }

        // Test multi-field sorting
        // First, make sure we have files with different sizes
        let mut entries = list_entries(
            &dir_path,
            &SelectionType::All,
            &None,
            None,
            true, // recursive to get more files
        )
        .unwrap();

        // Sort by is_dir (to group directories and files) then by name
        let order_by_multiple = vec![
            crate::parser::OrderTerm {
                field: "is_dir".to_string(),
                direction: OrderDirection::Descending, // Directories first
            },
            crate::parser::OrderTerm {
                field: "name".to_string(),
                direction: OrderDirection::Ascending,
            },
        ];

        sort_entries(&mut entries, &order_by_multiple);

        // Verify directories come first, then files, and each group is sorted by name
        let mut last_is_dir = true;
        let mut last_name = String::new();

        for entry in &entries {
            // Once we switch from directories to files, reset the name comparison
            if last_is_dir && !entry.is_dir {
                last_is_dir = false;
                last_name = String::new();
            }

            // Within the same group (dirs or files), names should be ascending
            if !last_name.is_empty() {
                if last_is_dir == entry.is_dir {
                    assert!(
                        last_name <= entry.name,
                        "Within same group, entries should be sorted by name ascending"
                    );
                }
            }

            last_name = entry.name.clone();
        }
    }
}
