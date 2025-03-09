# LSQL

LSQL is a command-line utility that queries files and directories using SQL-like syntax.

## Features

- Query your filesystem using a familiar SQL-like syntax
- Filter files and directories based on various properties (name, size, extension, etc.)
- Interactive shell mode with command history
- Colorized output
- Configurable logging levels for debugging
- Cross-platform (Windows, macOS, Linux)

## Installation

### From Source

To build from source, you'll need a Rust environment. If you don't have it yet, install it from [rustup.rs](https://rustup.rs/).

```bash
# Clone the repository
git clone https://github.com/yourusername/lsql.git
cd lsql

# Build and install
cargo install --path .
```

## Usage

### Basic Queries

List all files and directories in the current directory:

```bash
lsql "select * from .;"
```

List only files:

```bash
lsql "select files from .;"
# or the short form
lsql "select f from .;"
```

List only directories:

```bash
lsql "select directories from .;"
# or the short forms
lsql "select dirs from .;"
lsql "select d from .;"
```

### Filtering with WHERE

List all PNG files:

```bash
lsql "select * from . where ext = \"png\";"
```

List all files larger than 10MB:

```bash
lsql "select * from . where size > \"10mb\";"
```

List all hidden files:

```bash
lsql "select * from . where is_hidden = true;"
```

Combined conditions:

```bash
lsql "select * from . where ext = \"jpg\" and size > \"1mb\";"
```

### Interactive Shell

Start the interactive shell:

```bash
lsql shell
```

In shell mode, you can type queries directly without wrapping them in quotes:

```
lsql> select * from .;
lsql> select files from /tmp where size > "1mb";
```

### Options

- `-n, --no-color`: Disable colored output
- `-r, --recursive`: Enable recursive search
- `-l, --log-level <LEVEL>`: Set the logging level [possible values: off, error, warn, info, debug, trace] (default: info)
- `--help`: Show help information
- `--version`: Show version information

## Debugging

You can adjust the verbosity of the log output using the `--log-level` option:

```bash
# Show only errors
lsql --log-level error "select * from .;"

# Show detailed debug information
lsql --log-level debug "select files from .;"

# Maximum verbosity for troubleshooting
lsql --log-level trace "select * from . where ext = \"jpg\";"
```

## SQL-like Grammar

- `SELECT` - What to display (files, directories, or both)
  - `*`: All items
  - `files`, `f`: Only files
  - `directories`, `dirs`, `d`: Only directories
  - Specific fields: e.g., `select name, size, ext from .;`

- `FROM` - Where to search
  - Any valid path on your system
  - `.` for the current directory

- `WHERE` - Conditions for filtering
  - Properties: `name`, `path`, `size`, `ext`, `modified`, `created`, `is_hidden`, `is_dir`, `is_file`, `permissions`
  - Operators: `=`, `!=`, `<`, `<=`, `>`, `>=`, `like`, `contains`
  - Size units: `b`, `kb`, `mb`, `gb`, `tb`
  - Logical operators: `and`, `or`

## Contributing

Interested in contributing to LSQL? Check out the following resources:

- [CONTRIBUTING.md](CONTRIBUTING.md): Guide on how to add new features
- [Developer Guide](docs/developer_guide.md): Detailed explanation of the project architecture
- [Examples](docs/examples/): Step-by-step guides for implementing specific features
  - [Adding a LIMIT clause](docs/examples/add_limit_clause.md): Complete walkthrough of adding a LIMIT feature

The documentation covers how to:
- Add new SQL statements and clauses
- Implement new CLI commands
- Add new filtering conditions
- Follow the project's coding style

We welcome all contributions, from bug fixes to new features!

## License

MIT
