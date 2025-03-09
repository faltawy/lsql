# LSQL Themes Guide

This guide explains how to use and customize themes in LSQL.

## Introduction to Themes

LSQL themes allow you to customize the visual appearance of the application, including:

- Color schemes for different elements
- Border styles for tables
- Text formatting options
- And more

Themes are stored as TOML files, making them easy to create and edit.

## Built-in Themes

LSQL comes with several built-in themes:

### Default Theme

The standard theme with a clean, professional look:
- Blue headers
- Cyan directories
- White files
- Rounded borders

### Dark Theme

Optimized for dark terminal backgrounds:
- Bright blue headers
- Bright cyan directories
- Gray row alternation for better readability
- Rounded borders

### Light Theme

Optimized for light terminal backgrounds:
- Blue headers
- Blue directories
- Black file text (for better contrast)
- Light row alternation

### Vibrant Theme

High-contrast theme with bright colors:
- Magenta headers with cyan borders
- Bright cyan directories
- Bright white files
- Alternating row colors
- Thick borders

### Minimal Theme

Simplified theme with minimal styling:
- White headers
- Plain text without colors for files and directories
- ASCII borders (no Unicode)
- No Unicode symbols

## Using Themes

### Command Line Options

Select a theme using the `--theme` (or `-t`) option:

```bash
lsql --theme dark "select * from .;"
```

List all available themes:

```bash
lsql --list-themes
```

### Theme Commands

LSQL provides commands to manage themes:

#### List available themes

```bash
lsql theme list
```

#### Set a theme

```bash
lsql theme set --name dark
```

#### Create a new theme

```bash
lsql theme create --name mytheme --base dark --description "My custom theme"
```

This will create a new theme based on the dark theme, which you can then edit.

## Theme File Format

Themes are defined in TOML files with the following structure:

```toml
name = "themename"
description = "Theme description"

[colors]
# UI colors
prompt = "green"
error = "red"
warning = "yellow"
success = "green"
info = "blue"

# Table colors
header = "blue"
header_border = "blue"
row_odd = "gray"          # Optional
row_even = null           # Optional (use null for default/none)
border = "white"

# Entry type colors
directory = "cyan"
file = "white"
symlink = "magenta"
hidden = "gray"
executable = "green"

[styles]
bold_headers = true
italicize_paths = false
border_style = "rounded"   # "thick", "rounded", "double", "thin", "ascii"
use_unicode_symbols = true
```

### Colors

The following color names are supported:

#### Basic Colors
- `black`
- `red`
- `green`
- `yellow`
- `blue`
- `magenta`
- `cyan`
- `white`

#### Bright Variants
- `bright black` (or `gray`/`grey`)
- `bright red`
- `bright green`
- `bright yellow`
- `bright blue`
- `bright magenta`
- `bright cyan`
- `bright white`

### Border Styles

The following border styles are available:

- `thick`: Heavy UTF-8 borders
- `rounded`: Rounded UTF-8 borders
- `double`: Double-line UTF-8 borders
- `thin`: Standard thin UTF-8 borders
- `ascii`: Plain ASCII borders
- `ascii_rounded`: ASCII borders with rounded corners
- `none`: No borders

### Styles

- `bold_headers`: Whether to make table headers bold
- `italicize_paths`: Whether to italicize file and directory paths
- `use_unicode_symbols`: Whether to use Unicode symbols for file types

## Creating Custom Themes

### Location

Custom themes are stored in:
- Linux/macOS: `~/.config/lsql/themes/`
- Windows: `%APPDATA%\lsql\themes\`

### Steps to Create a Custom Theme

1. **Using the Command Line**:
   ```bash
   lsql theme create --name mytheme --base dark --description "My theme description"
   ```

2. **Manual Creation**:
   - Create a new file in the themes directory named `mytheme.toml`
   - Copy the structure from an existing theme
   - Customize the colors and styles

3. **Theme Testing**:
   ```bash
   lsql --theme mytheme "select * from .;"
   ```

## Color Combinations

Here are some effective color combinations for different terminal backgrounds:

### For Dark Terminals

1. **High Contrast**
   ```toml
   header = "bright white"
   border = "bright blue"
   directory = "bright cyan"
   file = "bright white"
   ```

2. **Subtle**
   ```toml
   header = "bright blue"
   border = "blue"
   directory = "cyan"
   file = "white"
   ```

### For Light Terminals

1. **High Contrast**
   ```toml
   header = "blue"
   border = "black"
   directory = "blue"
   file = "black"
   ```

2. **Subtle**
   ```toml
   header = "blue"
   border = "gray"
   directory = "blue"
   file = "black"
   ```

## Example Custom Themes

### Neon Theme

```toml
name = "neon"
description = "Bright cyberpunk-inspired theme"

[colors]
prompt = "bright green"
error = "bright red"
warning = "bright yellow"
success = "bright green"
info = "bright blue"
header = "bright magenta"
header_border = "bright green"
row_odd = "bright black"
row_even = null
border = "bright green"
directory = "bright cyan"
file = "bright white"
symlink = "bright magenta"
hidden = "bright black"
executable = "bright green"

[styles]
bold_headers = true
italicize_paths = false
border_style = "rounded"
use_unicode_symbols = true
```

### Classic Theme

```toml
name = "classic"
description = "Classic terminal look with minimal styling"

[colors]
prompt = "green"
error = "red"
warning = "yellow"
success = "green"
info = "blue"
header = "white"
header_border = "white"
row_odd = null
row_even = null
border = "white"
directory = "white"
file = "white"
symlink = "white"
hidden = "white"
executable = "white"

[styles]
bold_headers = true
italicize_paths = false
border_style = "ascii"
use_unicode_symbols = false
```

## Troubleshooting

1. **Theme not applying correctly**
   - Check that you spelled the theme name correctly
   - Try listing available themes with `lsql --list-themes`
   - Ensure your terminal supports the colors being used

2. **Custom theme not showing up**
   - Check the location of your theme file
   - Verify the TOML syntax is correct
   - Try using `--theme` with the full path to your theme file

3. **Colors not displaying**
   - Make sure you're not using `--no-color` option
   - Check that your terminal supports color
   - For Windows CMD, you may need to enable ANSI color support 