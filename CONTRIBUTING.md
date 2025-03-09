# Contributing to LSQL

Thank you for your interest in contributing to LSQL! This guide will help you understand the project structure and how to extend it with new features.

## Project Structure

LSQL is organized into several modules:

- `parser`: Handles parsing SQL-like queries using the PEST grammar
- `fs`: Manages file system operations and filtering
- `display`: Formats and displays the query results
- `cli`: Provides the command-line interface and interactive shell

## How to Add New Features

### Adding New SQL Commands and Statements

LSQL uses the PEST parser generator to define its grammar. Here's how to add new commands:

#### 1. Extend the Grammar

First, modify `src/parser/lsql.pest` to include your new syntax. For example, to add an `ORDER BY` clause:

```rust
// Add to the main query structure
query = { select_clause ~ from_clause ~ where_clause? ~ order_by_clause? ~ ";" }

// Define the new ORDER BY clause
order_by_clause = { "order" ~ "by" ~ order_field ~ order_direction? }
order_field = { identifier }
order_direction = { "asc" | "desc" }
```

#### 2. Update the Query Structure

Next, update the `Query` struct in `src/parser/mod.rs` to include your new feature:

```rust
#[derive(Debug, Clone)]
pub struct Query {
    pub selection: SelectionType,
    pub path: String,
    pub condition: Option<ConditionNode>,
    pub order_by: Option<OrderBy>,  // Add this line
}

// Define a new struct for the ORDER BY clause
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub field: String,
    pub ascending: bool,
}
```

#### 3. Update the Parser

Modify the `parse_query` method in `LSQLParser` to handle the new clause:

```rust
pub fn parse_query(query_str: &str) -> Result<Query, String> {
    // Existing code...
    
    let mut order_by = None;
    
    // Process each part of the query
    for pair in pairs {
        match pair.as_rule() {
            // Existing rules...
            Rule::order_by_clause => {
                order_by = Some(Self::parse_order_by(pair.into_inner()));
            }
            _ => {}
        }
    }
    
    Ok(Query {
        selection,
        path,
        condition,
        order_by,  // Add this line
    })
}

// Add a new parsing method
fn parse_order_by(mut pairs: Pairs<Rule>) -> OrderBy {
    let field_pair = pairs.next().unwrap();
    let field = field_pair.as_str().to_string();
    
    // Check for direction (defaults to ascending if not specified)
    let ascending = if let Some(dir_pair) = pairs.next() {
        dir_pair.as_str() != "desc"
    } else {
        true
    };
    
    OrderBy { field, ascending }
}
```

#### 4. Implement the New Functionality

In `fs.rs`, implement a function to apply the ordering:

```rust
// In fs.rs, update the list_entries function
pub fn list_entries(
    path: &str,
    selection: &SelectionType,
    condition: &Option<ConditionNode>,
    order_by: &Option<OrderBy>,  // Add this parameter
    recursive: bool,
) -> Result<Vec<FSEntry>, String> {
    // Existing code to get entries...
    
    // Apply ordering if specified
    if let Some(order) = order_by {
        entries.sort_by(|a, b| {
            let cmp = match order.field.as_str() {
                "name" => a.name.cmp(&b.name),
                "size" => a.size.cmp(&b.size),
                "modified" => a.modified.cmp(&b.modified),
                // Add more fields as needed
                _ => std::cmp::Ordering::Equal,
            };
            
            if order.ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }
    
    Ok(entries)
}
```

#### 5. Update the CLI

Finally, update the `execute_query` method in `cli.rs` to use the new functionality:

```rust
fn execute_query(&self, query_str: &str) -> Result<(), String> {
    // Existing code...
    
    let entries = fs::list_entries(
        &query.path,
        &query.selection,
        &query.condition,
        &query.order_by,  // Add this line
        self.recursive,
    )?;
    
    // Rest of the existing code...
}
```

### Adding New CLI Subcommands

To add a new subcommand to the CLI:

#### 1. Update the Command Enum

In `src/cli.rs`, add your new command to the `Command` enum:

```rust
#[derive(Subcommand, Clone)]
enum Command {
    /// Start interactive shell
    Shell,
    
    /// Your new command description
    Stats {
        /// Optional parameters for your command
        #[clap(long)]
        detailed: bool,
    },
}
```

#### 2. Implement the Command Handler

Add a new method to the `CLI` implementation:

```rust
fn run_stats_command(&self, detailed: bool) -> Result<(), String> {
    // Implement your command functionality here
    println!("Running stats command (detailed: {})", detailed);
    
    // Example implementation
    let entries = fs::list_entries(".", &SelectionType::All, &None, &None, true)?;
    let total_size: u64 = entries.iter().filter(|e| e.is_file).map(|e| e.size).sum();
    
    println!("Total files: {}", entries.iter().filter(|e| e.is_file).count());
    println!("Total directories: {}", entries.iter().filter(|e| e.is_dir).count());
    println!("Total size: {}", display::format_size(total_size));
    
    if detailed {
        // Add detailed output here
    }
    
    Ok(())
}
```

#### 3. Update the Main Run Method

Modify the `run` method to handle your new command:

```rust
pub fn run(self, args: Args) -> Result<(), String> {
    match args.command {
        Some(Command::Shell) => self.run_interactive_shell(),
        Some(Command::Stats { detailed }) => self.run_stats_command(detailed),
        None => match args.query {
            // Existing code...
        },
    }
}
```

### Adding New Filtering Conditions

To add new filtering capabilities:

#### 1. Update the Identifier List

In the PEST grammar (`src/parser/lsql.pest`), add your new field to the identifier list:

```rust
identifier = { 
    "name" | "path" | "size" | "modified" | "created" | "ext" | 
    "permissions" | "owner" | "is_hidden" | "is_readonly" | "your_new_field" 
}
```

#### 2. Update the FSEntry Struct

Add the new field to the `FSEntry` struct in `src/fs.rs`:

```rust
pub struct FSEntry {
    // Existing fields...
    pub your_new_field: YourType,
}
```

#### 3. Update the Evaluation Logic

Modify the `evaluate_single_condition` method in `FSEntry` to handle your new field:

```rust
fn evaluate_single_condition(
    &self,
    identifier: &str,
    operator: &ComparisonOperator,
    value: &Value,
) -> bool {
    match identifier {
        // Existing fields...
        "your_new_field" => {
            // Implement comparison logic for your new field
            match value {
                // Define comparison behavior based on the field type
            }
        }
        _ => {
            warn!("Unknown identifier in condition: {}", identifier);
            false
        }
    }
}
```

## Testing Your Changes

Always add tests for your new features:

1. **Unit Tests**: Add unit tests for each component in the respective module's `tests` section
2. **Integration Tests**: Add end-to-end tests to verify the functionality works as expected

Example unit test for a new ORDER BY feature:

```rust
#[test]
fn test_order_by_clause() {
    let query = "select * from . order by name desc;";
    let result = LSQLParser::parse_query(query).unwrap();
    
    assert!(result.order_by.is_some());
    let order_by = result.order_by.unwrap();
    assert_eq!(order_by.field, "name");
    assert_eq!(order_by.ascending, false);
}
```

## Coding Style

- Follow Rust's official style guidelines
- Use meaningful variable and function names
- Add appropriate comments and documentation
- Use proper error handling (prefer `Result` over `panic!`)
- Add logging at appropriate levels for debugging

## Submitting Changes

1. Fork the repository
2. Create a feature branch for your changes
3. Add tests for your new functionality
4. Update documentation (including README.md if necessary)
5. Submit a pull request

Thank you for contributing to LSQL! 