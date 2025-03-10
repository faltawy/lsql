// Display Module
// This module handles formatting and displaying query results to the user

use comfy_table::{Cell, ContentArrangement, Row, Table};
// Local is used indirectly through chrono's features
#[allow(unused_imports)]
use chrono::Local;

use crate::fs::FSEntry;
use crate::parser::SelectionType;
use crate::theme::{apply_color, get_border_style, string_to_table_color, Theme};

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
pub fn display_entries(
    entries: &[FSEntry],
    selection: &SelectionType,
    theme: &Theme,
    use_color: bool,
) -> String {
    let mut table = Table::new();

    // Apply theme's border style
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Apply borders based on theme style
    let border_style = get_border_style(&theme.styles.border_style);
    if border_style != "none" {
        // Apply some kind of border preset
        // Let's use simple borders for now since comfy-table doesn't expose all border types
        use comfy_table::presets::UTF8_BORDERS_ONLY;
        table.load_preset(UTF8_BORDERS_ONLY);
    }

    // Set header
    table.set_header(get_header_row(selection, theme, use_color));

    // Add rows with alternating background styling
    for entry in entries {
        let row = get_entry_row(entry, selection, theme, use_color);
        table.add_row(row);
    }

    table.to_string()
}

// Get the header row based on selection type
fn get_header_row(selection: &SelectionType, theme: &Theme, use_color: bool) -> Row {
    let mut row = Row::new();

    // Apply header color from theme
    let header_color = if use_color {
        string_to_table_color(&theme.colors.header)
    } else {
        None
    };

    // Create header cells with theme styling
    let create_header_cell = |text: &str| {
        let mut cell = Cell::new(text);

        if theme.styles.bold_headers {
            cell = cell.add_attribute(comfy_table::Attribute::Bold);
        }

        if let Some(color) = header_color {
            cell = cell.fg(color);
        }

        cell
    };

    // Default columns
    row.add_cell(create_header_cell("Name"));

    // Add additional columns based on selection
    match selection {
        SelectionType::All | SelectionType::Files | SelectionType::Directories => {
            row.add_cell(create_header_cell("Type"));
            row.add_cell(create_header_cell("Size"));
            row.add_cell(create_header_cell("Modified"));
            row.add_cell(create_header_cell("Permissions"));
        }
        SelectionType::Fields(fields) => {
            for field in fields {
                row.add_cell(create_header_cell(field));
            }
        }
    }

    row
}

// Get a row for a single entry
fn get_entry_row(
    entry: &FSEntry,
    selection: &SelectionType,
    theme: &Theme,
    use_color: bool,
) -> Row {
    let mut row = Row::new();

    // Determine color based on file type and theme
    let name_color = if !use_color {
        None
    } else if entry.is_dir {
        string_to_table_color(&theme.colors.directory)
    } else if entry.is_hidden {
        string_to_table_color(&theme.colors.hidden)
    } else {
        string_to_table_color(&theme.colors.file)
    };

    // Apply potential path italicization from theme
    let name_text = if theme.styles.italicize_paths {
        entry.name.to_string()
    } else {
        entry.name.clone()
    };

    // Create the cell with theme styling
    let name_cell = match name_color {
        Some(color) => Cell::new(&name_text).fg(color),
        None => Cell::new(&name_text),
    };

    row.add_cell(name_cell);

    // Add additional columns based on selection
    match selection {
        SelectionType::All | SelectionType::Files | SelectionType::Directories => {
            // Type column
            let type_str = if entry.is_dir { "dir" } else { "file" };
            row.add_cell(Cell::new(type_str));

            // Size column
            let size_str = if entry.is_dir {
                "-"
            } else {
                &format_size(entry.size)
            };
            row.add_cell(Cell::new(size_str));

            // Modified date column
            let date_str = entry.modified.format("%Y-%m-%d %H:%M").to_string();
            row.add_cell(Cell::new(&date_str));

            // Permissions column
            row.add_cell(Cell::new(&entry.permissions));
        }
        SelectionType::Fields(fields) => {
            for field in fields {
                match field.as_str() {
                    "name" => {} // Already added
                    "path" => {
                        // Apply italics to path if theme says so
                        let path_text = if theme.styles.italicize_paths && use_color {
                            entry.path.to_string()
                        } else {
                            entry.path.clone()
                        };
                        row.add_cell(Cell::new(&path_text));
                    }
                    "size" => {
                        row.add_cell(Cell::new(format_size(entry.size)));
                    }
                    "modified" => {
                        let date_str = entry.modified.format("%Y-%m-%d %H:%M").to_string();
                        row.add_cell(Cell::new(&date_str));
                    }
                    "created" => {
                        let date_str = entry.created.format("%Y-%m-%d %H:%M").to_string();
                        row.add_cell(Cell::new(&date_str));
                    }
                    "ext" => {
                        let ext_str = entry.extension.as_deref().unwrap_or("-");
                        row.add_cell(Cell::new(ext_str));
                    }
                    "permissions" => {
                        row.add_cell(Cell::new(&entry.permissions));
                    }
                    "is_hidden" => {
                        row.add_cell(Cell::new(entry.is_hidden.to_string()));
                    }
                    "is_dir" => {
                        row.add_cell(Cell::new(entry.is_dir.to_string()));
                    }
                    "is_file" => {
                        row.add_cell(Cell::new(entry.is_file.to_string()));
                    }
                    _ => {
                        row.add_cell(Cell::new("-"));
                    }
                }
            }
        }
    }

    row
}

// Format a message with theme
pub fn format_message(message: &str, color_name: &str, theme: &Theme, use_color: bool) -> String {
    if !use_color {
        return message.to_string();
    }

    let color = match color_name {
        "error" => &theme.colors.error,
        "warning" => &theme.colors.warning,
        "success" => &theme.colors.success,
        "info" => &theme.colors.info,
        _ => color_name,
    };

    apply_color(message, color, use_color).to_string()
}

// Display the results of a query
pub fn display_results(
    results: &[FSEntry],
    selection: &SelectionType,
    theme: &Theme,
    use_color: bool,
) -> Result<(), String> {
    if results.is_empty() {
        let message = format_message("No results found.", "warning", theme, use_color);
        println!("{}", message);
    } else {
        let table = display_entries(results, selection, theme, use_color);
        println!("{}", table);
    }

    Ok(())
}
