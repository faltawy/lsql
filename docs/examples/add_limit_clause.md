# Example: Adding a LIMIT Clause to LSQL

This guide provides a concrete example of how to add a new SQL feature to LSQL. We'll implement a `LIMIT` clause, which will restrict the number of results returned by a query.

## Step 1: Update the Grammar

First, we need to extend the PEST grammar in `src/parser/lsql.pest`:

```diff
 // Main query structure
-query = { select_clause ~ from_clause ~ where_clause? ~ ";" }
+query = { select_clause ~ from_clause ~ where_clause? ~ limit_clause? ~ ";" }
+
+// Define the LIMIT clause
+limit_clause = { "limit" ~ number }
```

## Step 2: Update the Query Structure

Next, we update the `Query` struct in `src/parser/mod.rs` to include the limit parameter:

```diff
 #[derive(Debug, Clone)]
 pub struct Query {
     pub selection: SelectionType,
     pub path: String,
     pub condition: Option<ConditionNode>,
+    pub limit: Option<u64>,
 }
```

## Step 3: Update the Parser

Modify the parser to handle the new limit clause:

```diff
 pub fn parse_query(query_str: &str) -> Result<Query, String> {
     // Parse the query using PEST
     let pairs = match LSQLParser::parse(Rule::query, query_str) {
         Ok(mut pairs) => pairs.next().unwrap().into_inner(),
         Err(e) => return Err(format!("Parse error: {}", e)),
     };

     let mut selection = SelectionType::All;
     let mut path = String::new();
     let mut condition = None;
+    let mut limit = None;
     
     // Process each part of the query
     for pair in pairs {
         match pair.as_rule() {
             Rule::select_clause => {
                 selection = Self::parse_selection(pair.into_inner());
             }
             Rule::from_clause => {
                 path = Self::parse_path(pair.into_inner());
             }
             Rule::where_clause => {
                 condition = Some(Self::parse_condition(pair.into_inner()));
             }
+            Rule::limit_clause => {
+                limit = Some(Self::parse_limit(pair.into_inner()));
+            }
             _ => {}
         }
     }
     
     Ok(Query {
         selection,
         path,
         condition,
+        limit,
     })
 }
 
+// Parse the limit clause
+fn parse_limit(mut pairs: Pairs<Rule>) -> u64 {
+    let limit_pair = pairs.next().unwrap();
+    let limit_str = limit_pair.as_str();
+    
+    match limit_str.parse::<u64>() {
+        Ok(n) => n,
+        Err(_) => {
+            warn!("Invalid limit value: {}, defaulting to 100", limit_str);
+            100 // Default to 100 if parsing fails
+        }
+    }
+}
```

## Step 4: Apply the Limit in the File System Module

Update the `list_entries` function in `src/fs.rs` to handle the limit parameter:

```diff
 pub fn list_entries(
     path: &str, 
     selection: &SelectionType, 
     condition: &Option<ConditionNode>,
+    limit: Option<u64>,
     recursive: bool
 ) -> Result<Vec<FSEntry>, String> {
     // Existing code...
     
     // Apply filtering as before
     let mut entries = vec![];
     
     for entry in walker.into_iter().filter_map(|e| e.ok()) {
         // Existing filtering code...
         
         // Add entries that match the criteria
         if include && fs_entry.matches_condition(condition) {
             entries.push(fs_entry);
+            
+            // Check if we've reached the limit
+            if let Some(limit_val) = limit {
+                if entries.len() >= limit_val as usize {
+                    debug!("Limit of {} entries reached, stopping search", limit_val);
+                    break;
+                }
+            }
         }
     }
     
     debug!("Found {} entries after filtering", entries.len());
     
     Ok(entries)
 }
```

## Step 5: Update the CLI Module

Finally, update the `execute_query` method in the CLI module to pass the limit parameter:

```diff
 fn execute_query(&self, query_str: &str) -> Result<(), String> {
     // Existing code...
     
     let entries = match fs::list_entries(
         &query.path,
         &query.selection,
         &query.condition,
+        query.limit,
         self.recursive,
     ) {
         // Existing code...
     };
     
     // Rest of the code remains the same...
 }
```

## Step 6: Add Tests

Add tests to verify the new functionality:

```rust
#[test]
fn test_limit_clause() {
    let query = "select * from . limit 5;";
    let result = LSQLParser::parse_query(query).unwrap();
    
    assert!(result.limit.is_some());
    let limit = result.limit.unwrap();
    assert_eq!(limit, 5);
}
```

## Step 7: Document the New Feature

Update the README.md file to include the new feature:

```diff
 ## SQL-like Grammar
 
 - `WHERE` - Conditions for filtering
   - Properties: `name`, `path`, `size`, `ext`, `modified`, `created`, `is_hidden`
   - Operators: `=`, `!=`, `<`, `<=`, `>`, `>=`, `like`, `contains`
   - Size units: `b`, `kb`, `mb`, `gb`, `tb`
   - Logical operators: `and`, `or`
+
+- `LIMIT` - Restrict the number of results
+  - Example: `select * from . limit 10;`
```

Add an example to the usage section:

```diff
 ### Examples
 
 List all PNG files:
 
 ```bash
 lsql "select * from . where ext = \"png\";"
 ```
+
+List only the first 10 files:
+
+```bash
+lsql "select * from . limit 10;"
+```
```

## Testing the Changes

To test your changes, build the project and try some queries with the LIMIT clause:

```bash
# Build the project
cargo build

# Test with a simple limit
cargo run -- "select * from . limit 3;"

# Test with a limit and a condition
cargo run -- "select files from . where size > \"1mb\" limit 2;"
```

## Conclusion

By following these steps, you've successfully added a new LIMIT clause to LSQL. The same pattern can be applied to implement other SQL features such as:

- `ORDER BY` for sorting results
- `GROUP BY` for grouping results
- `OFFSET` for pagination
- Custom functions and operators

Remember to follow the project's coding style and add proper documentation and tests for any new feature. 