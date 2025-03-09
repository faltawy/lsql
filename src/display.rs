// Display Module
// This module handles formatting and displaying query results to the user

use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
// Local is used indirectly through chrono's features
#[allow(unused_imports)]
use chrono::Local;

use crate::fs::FSEntry;
use crate::parser::SelectionType;

// Format file size in human-readable form
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size < TB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else {
        format!("{:.2} TB", size as f64 / TB as f64)
    }
}

// Create a table with file system entries
pub fn display_entries(entries: &[FSEntry], selection: &SelectionType, use_color: bool) -> String {
    let mut table = Table::new();

    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(get_header_row(selection, use_color));

    for entry in entries {
        table.add_row(get_entry_row(entry, selection, use_color));
    }

    table.to_string()
}

// Get the header row based on selection type
fn get_header_row(selection: &SelectionType, use_color: bool) -> Row {
    let mut row = Row::new();

    // Default columns
    row.add_cell(create_cell("Name", use_color, Some(Color::Blue)));

    // Add additional columns based on selection
    match selection {
        SelectionType::All | SelectionType::Files | SelectionType::Directories => {
            row.add_cell(create_cell("Type", use_color, Some(Color::Blue)));
            row.add_cell(create_cell("Size", use_color, Some(Color::Blue)));
            row.add_cell(create_cell("Modified", use_color, Some(Color::Blue)));
            row.add_cell(create_cell("Permissions", use_color, Some(Color::Blue)));
        }
        SelectionType::Fields(fields) => {
            for field in fields {
                row.add_cell(create_cell(field, use_color, Some(Color::Blue)));
            }
        }
    }

    row
}

// Get a row for a single entry
fn get_entry_row(entry: &FSEntry, selection: &SelectionType, use_color: bool) -> Row {
    let mut row = Row::new();

    // Name is always included, with coloring based on type
    let name_color = if entry.is_dir {
        Some(Color::Cyan)
    } else if entry.is_hidden {
        Some(Color::DarkGrey)
    } else {
        None
    };

    row.add_cell(create_cell(&entry.name, use_color, name_color));

    // Add additional columns based on selection
    match selection {
        SelectionType::All | SelectionType::Files | SelectionType::Directories => {
            // Type column
            let type_str = if entry.is_dir { "dir" } else { "file" };
            row.add_cell(create_cell(type_str, use_color, None));

            // Size column
            let size_str = if entry.is_dir {
                "-"
            } else {
                &format_size(entry.size)
            };
            row.add_cell(create_cell(size_str, use_color, None));

            // Modified date column
            let date_str = entry.modified.format("%Y-%m-%d %H:%M").to_string();
            row.add_cell(create_cell(&date_str, use_color, None));

            // Permissions column
            row.add_cell(create_cell(&entry.permissions, use_color, None));
        }
        SelectionType::Fields(fields) => {
            for field in fields {
                match field.as_str() {
                    "name" => {} // Already added
                    "path" => {
                        row.add_cell(create_cell(&entry.path, use_color, None));
                    }
                    "size" => {
                        row.add_cell(create_cell(&format_size(entry.size), use_color, None));
                    }
                    "modified" => {
                        let date_str = entry.modified.format("%Y-%m-%d %H:%M").to_string();
                        row.add_cell(create_cell(&date_str, use_color, None));
                    }
                    "created" => {
                        let date_str = entry.created.format("%Y-%m-%d %H:%M").to_string();
                        row.add_cell(create_cell(&date_str, use_color, None));
                    }
                    "ext" => {
                        let ext_str = entry.extension.as_deref().unwrap_or("-");
                        row.add_cell(create_cell(ext_str, use_color, None));
                    }
                    "permissions" => {
                        row.add_cell(create_cell(&entry.permissions, use_color, None));
                    }
                    "is_hidden" => {
                        row.add_cell(create_cell(&entry.is_hidden.to_string(), use_color, None));
                    }
                    "is_dir" => {
                        row.add_cell(create_cell(&entry.is_dir.to_string(), use_color, None));
                    }
                    "is_file" => {
                        row.add_cell(create_cell(&entry.is_file.to_string(), use_color, None));
                    }
                    _ => {
                        row.add_cell(create_cell("-", use_color, None));
                    }
                }
            }
        }
    }

    row
}

// Create a cell with optional color
fn create_cell(content: &str, use_color: bool, color: Option<Color>) -> Cell {
    let mut cell = Cell::new(content);

    if use_color {
        if let Some(c) = color {
            cell = cell.fg(c);
        }
    }

    cell
}
