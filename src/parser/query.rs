// Query parsing module
use super::condition;
use super::selection;
use super::types::{ConditionNode, LSQLParser, Pairs, Rule, SelectionType};
use log::{debug, trace, warn};
use pest::Parser;

/// Represents a parsed query
#[derive(Debug, Clone)]
pub struct Query {
    /// What to select (files, directories, or specific fields)
    pub selection: SelectionType,
    /// Path to search in
    pub path: String,
    /// Optional condition for filtering
    pub condition: Option<ConditionNode>,
    /// Optional limit for restricting the number of results
    pub limit: Option<u64>,
}

impl LSQLParser {
    /// Parse a query string into a Query struct
    pub fn parse_query(query_str: &str) -> Result<Query, String> {
        debug!("Parsing query: {}", query_str);

        // Parse the query using PEST
        let pairs = match LSQLParser::parse(Rule::query, query_str) {
            Ok(mut pairs) => pairs.next().unwrap().into_inner(),
            Err(e) => {
                warn!("Parse error: {}", e);
                return Err(format!("Parse error: {}", e));
            }
        };

        let mut selection = SelectionType::All;
        let mut path = String::new();
        let mut condition = None;
        let mut limit = None;

        // Process each part of the query
        for pair in pairs {
            trace!("Processing rule: {:?}", pair.as_rule());

            match pair.as_rule() {
                Rule::select_clause => {
                    debug!("Found select_clause: {}", pair.as_str());
                    selection = selection::parse_selection(pair.into_inner());
                }
                Rule::from_clause => {
                    debug!("Found from_clause: {}", pair.as_str());
                    path = parse_path(pair.into_inner());
                }
                Rule::where_clause => {
                    debug!("Found where_clause: {}", pair.as_str());
                    condition = Some(condition::parse_condition(pair.into_inner()));
                }
                Rule::limit_clause => {
                    debug!("Found limit_clause: {}", pair.as_str());
                    limit = Some(parse_limit(pair.into_inner()));
                }
                _ => {
                    trace!("Found unknown rule: {}", pair.as_str());
                }
            }
        }

        debug!(
            "Parsed query: selection={:?}, path={}, condition={}, limit={}",
            selection,
            path,
            if condition.is_some() {
                "present"
            } else {
                "none"
            },
            if let Some(l) = limit {
                l.to_string()
            } else {
                "none".to_string()
            }
        );

        Ok(Query {
            selection,
            path,
            condition,
            limit,
        })
    }
}

/// Parse the path part of the query
fn parse_path(mut pairs: Pairs<Rule>) -> String {
    // Get the path pair
    if let Some(path_pair) = pairs.next() {
        let path_str = path_pair.as_str();
        trace!("Path string: '{}'", path_str);

        // Remove quotes if present
        let result = if path_str.starts_with('"') && path_str.ends_with('"') {
            path_str[1..path_str.len() - 1].to_string()
        } else {
            path_str.to_string()
        };

        debug!("Parsed path: '{}'", result);
        result
    } else {
        // Default to current directory if no path specified
        debug!("No path specified, defaulting to current directory '.'");
        ".".to_string()
    }
}

/// Parse the limit part of the query
fn parse_limit(mut pairs: Pairs<Rule>) -> u64 {
    // Get the number pair
    if let Some(number_pair) = pairs.next() {
        let number_str = number_pair.as_str();
        trace!("Limit number string: '{}'", number_str);

        match number_str.parse::<u64>() {
            Ok(n) => {
                debug!("Parsed limit: {}", n);
                n
            }
            Err(_) => {
                warn!("Invalid limit value: {}, defaulting to 100", number_str);
                100 // Default to 100 if parsing fails
            }
        }
    } else {
        // Default to 100 if no limit specified
        debug!("No limit specified, defaulting to 100");
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LogicalOperator;

    #[test]
    fn test_basic_query() {
        let query = "select * from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_none());
    }

    #[test]
    fn test_files_only_query() {
        let query = "select files from /tmp;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(matches!(result.selection, SelectionType::Files));
        assert_eq!(result.path, "/tmp");
    }

    #[test]
    fn test_directories_only_query() {
        let query = "select directories from /home;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(matches!(result.selection, SelectionType::Directories));
        assert_eq!(result.path, "/home");
    }

    #[test]
    fn test_field_list_selection() {
        let query = "select name, size, ext from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 3);
            assert!(fields.contains(&"name".to_string()));
            assert!(fields.contains(&"size".to_string()));
            assert!(fields.contains(&"ext".to_string()));
        } else {
            panic!("Expected Fields selection");
        }
    }

    #[test]
    fn test_quoted_path() {
        let query = "select * from \"path with spaces\";";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.path, "path with spaces");
    }

    #[test]
    fn test_optional_semicolon() {
        // With semicolon
        let query_with_semicolon = "select * from .;";
        let result_with_semicolon = LSQLParser::parse_query(query_with_semicolon);
        assert!(result_with_semicolon.is_ok());

        // Without semicolon
        let query_without_semicolon = "select * from .";
        let result_without_semicolon = LSQLParser::parse_query(query_without_semicolon);
        assert!(result_without_semicolon.is_ok());

        // With condition and with semicolon
        let query_cond_with_semicolon = "select * from . where name = \"test\";";
        let result_cond_with_semicolon = LSQLParser::parse_query(query_cond_with_semicolon);
        assert!(result_cond_with_semicolon.is_ok());

        // With condition and without semicolon
        let query_cond_without_semicolon = "select * from . where name = \"test\"";
        let result_cond_without_semicolon = LSQLParser::parse_query(query_cond_without_semicolon);
        assert!(result_cond_without_semicolon.is_ok());
    }

    #[test]
    fn test_invalid_queries() {
        // Incomplete query (missing from clause)
        let query1 = "select *";
        let result1 = LSQLParser::parse_query(query1);
        assert!(
            result1.is_err(),
            "Query without FROM clause should be invalid"
        );

        // Completely invalid syntax with missing required elements
        let query2 = "selecty & fromy $ wherey @";
        let result2 = LSQLParser::parse_query(query2);
        assert!(result2.is_err(), "Gibberish query should be invalid");
    }

    #[test]
    fn test_complex_query() {
        let query =
            "select files from . where (size > 1mb and is_hidden = false) or ext = \"pdf\";";
        let result = LSQLParser::parse_query(query).unwrap();

        // Test selection is Files
        assert!(matches!(result.selection, SelectionType::Files));

        // Test path is current directory
        assert_eq!(result.path, ".");

        // Verify that we have a condition
        assert!(result.condition.is_some());

        // Check for a branch with OR at the top level
        let top_level_or = match &result.condition {
            Some(ConditionNode::Branch { operator, .. }) => {
                matches!(operator, LogicalOperator::Or)
            }
            _ => false,
        };

        assert!(top_level_or, "Expected a top-level OR branch");
    }

    #[test]
    fn test_limit_clause() {
        // Basic limit clause
        let query = "select * from . limit 5;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(result.limit.is_some());
        let limit = result.limit.unwrap();
        assert_eq!(limit, 5);

        // Test with a condition and limit
        let query_with_condition = "select files from . where size > 1mb limit 10;";
        let result_with_condition = LSQLParser::parse_query(query_with_condition).unwrap();

        assert!(result_with_condition.limit.is_some());
        let limit_with_condition = result_with_condition.limit.unwrap();
        assert_eq!(limit_with_condition, 10);

        // Test with zero limit
        let query_zero_limit = "select * from . limit 0;";
        let result_zero_limit = LSQLParser::parse_query(query_zero_limit).unwrap();

        assert!(result_zero_limit.limit.is_some());
        let zero_limit = result_zero_limit.limit.unwrap();
        assert_eq!(zero_limit, 0);

        // Test with large limit
        let query_large_limit = "select * from . limit 1000000;";
        let result_large_limit = LSQLParser::parse_query(query_large_limit).unwrap();

        assert!(result_large_limit.limit.is_some());
        let large_limit = result_large_limit.limit.unwrap();
        assert_eq!(large_limit, 1000000);

        // Test without limit clause
        let query_no_limit = "select * from .;";
        let result_no_limit = LSQLParser::parse_query(query_no_limit).unwrap();

        assert!(result_no_limit.limit.is_none());
    }
}
