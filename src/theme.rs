// Theme Module
// This module handles all theming and color customization for the application

use colored::{ColoredString, Colorize};
use comfy_table::Color;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Define a struct to represent a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
}

// Define colors used in the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // General UI colors
    pub prompt: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub info: String,

    // Table colors
    pub header: String,
    pub header_border: String,
    pub row_odd: Option<String>,
    pub row_even: Option<String>,
    pub border: String,

    // Entry type colors
    pub directory: String,
    pub file: String,
    pub symlink: String,
    pub hidden: String,
    pub executable: String,
}

// Define text styles used in the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeStyles {
    pub bold_headers: bool,
    pub italicize_paths: bool,
    pub border_style: String, // "thick", "thin", "rounded", etc.
    pub use_unicode_symbols: bool,
}

// Default themes
impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "default".to_string(),
            description: "Default LSQL theme with a clean, modern look".to_string(),
            colors: ThemeColors::default(),
            styles: ThemeStyles::default(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors {
            // General UI colors
            prompt: "green".to_string(),
            error: "red".to_string(),
            warning: "yellow".to_string(),
            success: "green".to_string(),
            info: "blue".to_string(),

            // Table colors
            header: "blue".to_string(),
            header_border: "blue".to_string(),
            row_odd: None,
            row_even: None,
            border: "white".to_string(),

            // Entry type colors
            directory: "cyan".to_string(),
            file: "white".to_string(), // Default, no coloring
            symlink: "magenta".to_string(),
            hidden: "bright black".to_string(), // bright black is actually gray
            executable: "green".to_string(),
        }
    }
}

impl Default for ThemeStyles {
    fn default() -> Self {
        ThemeStyles {
            bold_headers: true,
            italicize_paths: false,
            border_style: "rounded".to_string(),
            use_unicode_symbols: true,
        }
    }
}

// Built-in themes
pub fn dark_theme() -> Theme {
    Theme {
        name: "dark".to_string(),
        description: "Dark theme optimized for dark backgrounds".to_string(),
        colors: ThemeColors {
            // General UI colors
            prompt: "bright green".to_string(),
            error: "bright red".to_string(),
            warning: "bright yellow".to_string(),
            success: "bright green".to_string(),
            info: "bright blue".to_string(),

            // Table colors
            header: "bright blue".to_string(),
            header_border: "bright blue".to_string(),
            row_odd: Some("bright black".to_string()), // gray
            row_even: None,
            border: "white".to_string(),

            // Entry type colors
            directory: "bright cyan".to_string(),
            file: "white".to_string(),
            symlink: "bright magenta".to_string(),
            hidden: "bright black".to_string(),
            executable: "bright green".to_string(),
        },
        styles: ThemeStyles {
            bold_headers: true,
            italicize_paths: true,
            border_style: "rounded".to_string(),
            use_unicode_symbols: true,
        },
    }
}

pub fn light_theme() -> Theme {
    Theme {
        name: "light".to_string(),
        description: "Light theme optimized for light backgrounds".to_string(),
        colors: ThemeColors {
            // General UI colors
            prompt: "green".to_string(),
            error: "red".to_string(),
            warning: "yellow".to_string(),
            success: "green".to_string(),
            info: "blue".to_string(),

            // Table colors
            header: "blue".to_string(),
            header_border: "blue".to_string(),
            row_odd: Some("bright white".to_string()),
            row_even: None,
            border: "black".to_string(),

            // Entry type colors
            directory: "blue".to_string(),
            file: "black".to_string(),
            symlink: "magenta".to_string(),
            hidden: "gray".to_string(),
            executable: "green".to_string(),
        },
        styles: ThemeStyles {
            bold_headers: true,
            italicize_paths: false,
            border_style: "rounded".to_string(),
            use_unicode_symbols: true,
        },
    }
}

pub fn vibrant_theme() -> Theme {
    Theme {
        name: "vibrant".to_string(),
        description: "Vibrant theme with high contrast and bright colors".to_string(),
        colors: ThemeColors {
            // General UI colors
            prompt: "bright green".to_string(),
            error: "bright red".to_string(),
            warning: "bright yellow".to_string(),
            success: "bright green".to_string(),
            info: "bright blue".to_string(),

            // Table colors
            header: "bright magenta".to_string(),
            header_border: "bright cyan".to_string(),
            row_odd: Some("black".to_string()),
            row_even: Some("bright black".to_string()),
            border: "bright yellow".to_string(),

            // Entry type colors
            directory: "bright cyan".to_string(),
            file: "bright white".to_string(),
            symlink: "bright magenta".to_string(),
            hidden: "gray".to_string(),
            executable: "bright green".to_string(),
        },
        styles: ThemeStyles {
            bold_headers: true,
            italicize_paths: true,
            border_style: "thick".to_string(),
            use_unicode_symbols: true,
        },
    }
}

pub fn minimal_theme() -> Theme {
    Theme {
        name: "minimal".to_string(),
        description: "Minimal theme with limited colors and simple design".to_string(),
        colors: ThemeColors {
            // General UI colors
            prompt: "white".to_string(),
            error: "red".to_string(),
            warning: "yellow".to_string(),
            success: "green".to_string(),
            info: "blue".to_string(),

            // Table colors
            header: "white".to_string(),
            header_border: "white".to_string(),
            row_odd: None,
            row_even: None,
            border: "white".to_string(),

            // Entry type colors
            directory: "white".to_string(),
            file: "white".to_string(),
            symlink: "white".to_string(),
            hidden: "white".to_string(),
            executable: "white".to_string(),
        },
        styles: ThemeStyles {
            bold_headers: true,
            italicize_paths: false,
            border_style: "ascii".to_string(),
            use_unicode_symbols: false,
        },
    }
}

// Theme manager to handle all themes
#[derive(Debug, Clone)]
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
    user_config_dir: Option<PathBuf>,
}

impl ThemeManager {
    // Create a new theme manager with default themes
    pub fn new() -> Self {
        let mut available_themes = HashMap::new();

        // Add built-in themes
        let default_theme = Theme::default();
        let dark = dark_theme();
        let light = light_theme();
        let vibrant = vibrant_theme();
        let minimal = minimal_theme();

        available_themes.insert(default_theme.name.clone(), default_theme.clone());
        available_themes.insert(dark.name.clone(), dark);
        available_themes.insert(light.name.clone(), light);
        available_themes.insert(vibrant.name.clone(), vibrant);
        available_themes.insert(minimal.name.clone(), minimal);

        // Get user config directory
        let user_config_dir = dirs::config_dir().map(|dir| dir.join("lsql").join("themes"));

        ThemeManager {
            current_theme: default_theme,
            available_themes,
            user_config_dir,
        }
    }

    // Initialize the theme manager and load user themes
    pub fn initialize(&mut self) {
        // Load user themes if available
        if let Some(config_dir) = self.user_config_dir.clone() {
            // Create directory if it doesn't exist
            if !config_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&config_dir) {
                    warn!("Failed to create theme directory: {}", e);
                } else {
                    // Save built-in themes as examples
                    self.save_builtin_themes(&config_dir);
                }
            }

            // Load user themes
            self.load_user_themes(&config_dir);
        }
    }

    // Save built-in themes as examples in the user config directory
    fn save_builtin_themes(&self, config_dir: &PathBuf) {
        for theme in self.available_themes.values() {
            let theme_path = config_dir.join(format!("{}.toml", theme.name));

            if !theme_path.exists() {
                match toml::to_string_pretty(theme) {
                    Ok(toml_str) => {
                        if let Err(e) = fs::write(&theme_path, toml_str) {
                            warn!("Failed to write theme file {}: {}", theme_path.display(), e);
                        } else {
                            debug!("Saved built-in theme to {}", theme_path.display());
                        }
                    }
                    Err(e) => warn!("Failed to serialize theme {}: {}", theme.name, e),
                }
            }
        }
    }

    // Load user-defined themes from the config directory
    fn load_user_themes(&mut self, config_dir: &PathBuf) {
        if !config_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                // Only load .toml files
                if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                    match fs::read_to_string(&path) {
                        Ok(contents) => match toml::from_str::<Theme>(&contents) {
                            Ok(theme) => {
                                debug!("Loaded user theme: {}", theme.name);
                                self.available_themes.insert(theme.name.clone(), theme);
                            }
                            Err(e) => {
                                warn!("Failed to parse theme file {}: {}", path.display(), e);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to read theme file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    // Set the current theme by name
    pub fn set_theme(&mut self, theme_name: &str) -> Result<(), String> {
        if let Some(theme) = self.available_themes.get(theme_name) {
            self.current_theme = theme.clone();
            debug!("Set theme to: {}", theme_name);
            Ok(())
        } else {
            let error_msg = format!("Theme '{}' not found", theme_name);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }

    // Get a list of available theme names
    pub fn list_themes(&self) -> Vec<String> {
        self.available_themes.keys().cloned().collect()
    }

    // Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    // Create a new theme from scratch and save it
    pub fn create_theme(&mut self, theme: Theme) -> Result<(), String> {
        // Validate the theme
        if theme.name.is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }

        // Add to available themes
        self.available_themes
            .insert(theme.name.clone(), theme.clone());

        // Save to disk if possible
        if let Some(config_dir) = &self.user_config_dir {
            if !config_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(config_dir) {
                    return Err(format!("Failed to create theme directory: {}", e));
                }
            }

            let theme_path = config_dir.join(format!("{}.toml", theme.name));

            match toml::to_string_pretty(&theme) {
                Ok(toml_str) => {
                    if let Err(e) = fs::write(&theme_path, toml_str) {
                        return Err(format!("Failed to write theme file: {}", e));
                    }
                }
                Err(e) => return Err(format!("Failed to serialize theme: {}", e)),
            }
        }

        Ok(())
    }
}

// Helper functions to apply theme colors to strings
pub fn apply_color(text: &str, color_name: &str, use_color: bool) -> ColoredString {
    if !use_color {
        return text.normal();
    }

    match color_name.to_lowercase().as_str() {
        "black" => text.black(),
        "red" => text.red(),
        "green" => text.green(),
        "yellow" => text.yellow(),
        "blue" => text.blue(),
        "magenta" => text.magenta(),
        "cyan" => text.cyan(),
        "white" => text.white(),
        "bright black" | "gray" | "grey" => text.bright_black(),
        "bright red" => text.bright_red(),
        "bright green" => text.bright_green(),
        "bright yellow" => text.bright_yellow(),
        "bright blue" => text.bright_blue(),
        "bright magenta" => text.bright_magenta(),
        "bright cyan" => text.bright_cyan(),
        "bright white" => text.bright_white(),
        _ => text.normal(),
    }
}

// Convert color name string to comfy_table Color enum
pub fn string_to_table_color(color_name: &str) -> Option<Color> {
    match color_name.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "bright black" | "gray" | "grey" => Some(Color::DarkGrey),
        "bright red" => Some(Color::Red),
        "bright green" => Some(Color::Green),
        "bright yellow" => Some(Color::Yellow),
        "bright blue" => Some(Color::Blue),
        "bright magenta" => Some(Color::Magenta),
        "bright cyan" => Some(Color::Cyan),
        "bright white" => Some(Color::Grey),
        _ => None,
    }
}

// Get a border based on the theme's border style
pub fn get_border_style(style_name: &str) -> &str {
    // Since we can't easily use comfy-table's border styles directly,
    // we'll just return a string description of the border style
    match style_name.to_lowercase().as_str() {
        "thick" => "thick",
        "rounded" => "rounded",
        "double" => "double",
        "thin" => "thin",
        "ascii" => "ascii",
        "ascii_rounded" => "ascii_rounded",
        "none" => "none",
        _ => "thin", // Default to thin borders
    }
}
