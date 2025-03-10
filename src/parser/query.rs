// Query parsing module
use super::condition;
use super::selection;
use super::types::{
    ConditionNode, LSQLParser, OrderDirection, OrderTerm, Pairs, QueryType, Rule, SelectionType,
};
use log::{debug, trace, warn};
use pest::Parser;

/// Represents a parsed query
#[derive(Debug, Clone)]
pub struct Query {
    /// Type of query (SELECT or DELETE)
    pub query_type: QueryType,
    /// What to select (files, directories, or specific fields)
    pub selection: SelectionType,
    /// Path to search in
    pub path: String,
    /// Optional condition for filtering
    pub condition: Option<ConditionNode>,
    /// Optional limit for restricting the number of results
    pub limit: Option<u64>,
    /// Whether to perform recursive operations (for DELETE queries)
    pub is_recursive: bool,
    /// Optional order by terms for sorting results
    pub order_by: Vec<OrderTerm>,
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

        let mut query_type = QueryType::Select; // Default to SELECT
        let mut selection = SelectionType::All;
        let mut path = String::new();
        let mut condition = None;
        let mut limit = None;
        let mut is_recursive = false;
        let mut order_by = Vec::new();

        // Process each part of the query
        for pair in pairs {
            trace!("Processing rule: {:?}", pair.as_rule());

            match pair.as_rule() {
                Rule::select_query => {
                    debug!("Found select_query");
                    query_type = QueryType::Select;

                    // Process the select query parts
                    for select_part in pair.into_inner() {
                        match select_part.as_rule() {
                            Rule::select_clause => {
                                debug!("Found select_clause: {}", select_part.as_str());
                                selection = selection::parse_selection(select_part.into_inner());
                            }
                            Rule::from_clause => {
                                debug!("Found from_clause: {}", select_part.as_str());
                                path = parse_path(select_part.into_inner());
                            }
                            Rule::where_clause => {
                                debug!("Found where_clause: {}", select_part.as_str());
                                condition =
                                    Some(condition::parse_condition(select_part.into_inner()));
                            }
                            Rule::order_by_clause => {
                                debug!("Found order_by_clause: {}", select_part.as_str());
                                order_by = parse_order_by(select_part.into_inner());
                            }
                            Rule::limit_clause => {
                                debug!("Found limit_clause: {}", select_part.as_str());
                                limit = Some(parse_limit(select_part.into_inner()));
                            }
                            _ => {
                                trace!(
                                    "Found unknown rule in select_query: {}",
                                    select_part.as_str()
                                );
                            }
                        }
                    }
                }
                Rule::delete_query => {
                    debug!("Found delete_query");
                    query_type = QueryType::Delete;

                    // Process the delete query parts
                    for delete_part in pair.into_inner() {
                        match delete_part.as_rule() {
                            Rule::delete_clause => {
                                debug!("Found delete_clause: {}", delete_part.as_str());

                                // Check for recursive flag in the delete clause
                                for inner_part in delete_part.clone().into_inner() {
                                    if inner_part.as_rule() == Rule::recursive_flag {
                                        debug!("Found recursive flag in delete clause");
                                        is_recursive = true;
                                        break;
                                    }
                                }

                                // Parse the delete selection
                                let (sel, lim) = parse_delete_selection(delete_part.into_inner());
                                selection = sel;

                                // If a limit was specified in the delete selection, use it
                                if lim.is_some() {
                                    limit = lim;
                                }
                            }
                            Rule::from_clause => {
                                debug!("Found from_clause: {}", delete_part.as_str());
                                path = parse_path(delete_part.into_inner());
                            }
                            Rule::where_clause => {
                                debug!("Found where_clause: {}", delete_part.as_str());
                                condition =
                                    Some(condition::parse_condition(delete_part.into_inner()));
                            }
                            Rule::limit_clause => {
                                debug!("Found limit_clause: {}", delete_part.as_str());
                                // Only set the limit if it wasn't already set in the delete selection
                                if limit.is_none() {
                                    limit = Some(parse_limit(delete_part.into_inner()));
                                }
                            }
                            _ => {
                                trace!(
                                    "Found unknown rule in delete_query: {}",
                                    delete_part.as_str()
                                );
                            }
                        }
                    }
                }
                _ => {
                    trace!("Found unknown rule at top level: {}", pair.as_str());
                }
            }
        }

        debug!(
            "Parsed query: type={:?}, selection={:?}, path={}, condition={}, limit={}, recursive={}, order_by={}",
            query_type,
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
            },
            is_recursive,
            if order_by.is_empty() {
                "none".to_string()
            } else {
                format!("{} terms", order_by.len())
            }
        );

        Ok(Query {
            query_type,
            selection,
            path,
            condition,
            limit,
            is_recursive,
            order_by,
        })
    }
}

/// Parse the path part of the query
fn parse_path(pairs: Pairs<Rule>) -> String {
    // Extract the path from the pairs
    for pair in pairs {
        if pair.as_rule() == Rule::path {
            let path_str = pair.as_str();

            // Remove quotes if present
            if path_str.starts_with('"') && path_str.ends_with('"') {
                return path_str[1..path_str.len() - 1].to_string();
            } else {
                return path_str.to_string();
            }
        }
    }

    // Default to current directory if no path found
    ".".to_string()
}

/// Parse the limit part of the query
fn parse_limit(pairs: Pairs<Rule>) -> u64 {
    // Extract the number from the pairs
    for pair in pairs {
        if pair.as_rule() == Rule::number {
            let number_str = pair.as_str();

            // Parse the number, ignoring any size units
            let number_only = number_str
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect::<String>();

            match number_only.parse::<u64>() {
                Ok(n) => return n,
                Err(_) => {
                    warn!("Invalid limit value: {}, defaulting to 100", number_str);
                    return 100; // Default to 100 if parsing fails
                }
            }
        }
    }

    // Default to 100 if no number found
    100
}

/// Parse the delete selection part of the query
fn parse_delete_selection(pairs: Pairs<Rule>) -> (SelectionType, Option<u64>) {
    let mut selection = SelectionType::All;
    let mut limit = None;

    debug!("Parsing delete selection");

    for pair in pairs {
        debug!("Delete selection pair rule: {:?}", pair.as_rule());
        debug!("Delete selection pair text: {}", pair.as_str());

        if pair.as_rule() == Rule::recursive_flag {
            // Skip recursive flag, it's handled elsewhere
            continue;
        }

        if pair.as_rule() == Rule::delete_selection {
            let inner_pairs = pair.into_inner();
            for inner_pair in inner_pairs {
                debug!("Inner pair rule: {:?}", inner_pair.as_rule());
                debug!("Inner pair text: {}", inner_pair.as_str());

                match inner_pair.as_rule() {
                    Rule::FIRST => {
                        debug!("Found FIRST keyword");
                        // If FIRST is specified, set limit to 1 by default
                        limit = Some(1);
                    }
                    Rule::MANY => {
                        debug!("Found MANY keyword");
                        // If MANY is specified without a number, don't set a limit
                        // The limit might be specified separately in a LIMIT clause
                    }
                    Rule::number => {
                        debug!("Found number: {}", inner_pair.as_str());
                        // If a number is specified after FIRST or MANY, use it as the limit
                        let number_str = inner_pair.as_str();
                        if let Ok(n) = number_str.parse::<u64>() {
                            limit = Some(n);
                        }
                    }
                    _ => {
                        // For backward compatibility, handle * and field lists
                        if inner_pair.as_str() == "*" {
                            debug!("Found * selection");
                            selection = SelectionType::All;
                        } else {
                            debug!("Found field list: {}", inner_pair.as_str());
                            // This would be a field list
                            let fields: Vec<String> = inner_pair
                                .into_inner()
                                .map(|p| p.as_str().to_string())
                                .collect();
                            selection = SelectionType::Fields(fields);
                        }
                    }
                }
            }
        } else if pair.as_rule() == Rule::selection {
            // For backward compatibility, handle the old selection syntax
            debug!("Using old selection syntax");
            selection = selection::parse_selection(Pairs::single(pair));
        } else {
            debug!("Unexpected rule in delete selection: {:?}", pair.as_rule());
        }
    }

    debug!(
        "Parsed delete selection: selection={:?}, limit={:?}",
        selection, limit
    );

    (selection, limit)
}

/// Parse the order by clause
fn parse_order_by(mut pairs: Pairs<Rule>) -> Vec<OrderTerm> {
    let mut order_by = Vec::new();

    while let Some(order_term_pair) = pairs.next() {
        trace!("Processing order_term: {:?}", order_term_pair.as_rule());

        match order_term_pair.as_rule() {
            Rule::order_term => {
                let term = parse_order_term(order_term_pair.into_inner());
                order_by.push(term);
            }
            _ => {
                trace!(
                    "Found unknown rule in order_by_clause: {}",
                    order_term_pair.as_str()
                );
            }
        }
    }

    order_by
}

/// Parse an order term
fn parse_order_term(mut pairs: Pairs<Rule>) -> OrderTerm {
    let mut field = String::new();
    let mut direction = OrderDirection::Ascending;

    while let Some(part) = pairs.next() {
        trace!("Processing order_term_part: {:?}", part.as_rule());

        match part.as_rule() {
            Rule::field => {
                debug!("Found field: {}", part.as_str());
                field = part.as_str().to_string();
            }
            Rule::order_direction => {
                debug!("Found direction: {}", part.as_str());
                let dir_str = part.as_str().to_lowercase();
                direction = match dir_str.as_str() {
                    "asc" => OrderDirection::Ascending,
                    "desc" => OrderDirection::Descending,
                    _ => {
                        warn!("Unknown direction: {}, defaulting to Ascending", dir_str);
                        OrderDirection::Ascending
                    }
                };
            }
            _ => {
                trace!("Found unknown rule in order_term: {}", part.as_str());
            }
        }
    }

    OrderTerm { field, direction }
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
        let query = "select type from /tmp;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }
        assert_eq!(result.path, "/tmp");
    }

    #[test]
    fn test_directories_only_query() {
        let query = "select type from /home;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }
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
        let query = "select type from . where (size > 1mb and is_hidden = false) or ext = \"pdf\";";
        let result = LSQLParser::parse_query(query).unwrap();

        // Test selection is type field
        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }

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
        let query_with_condition = "select type from . where size > 1mb limit 10;";
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

    #[test]
    fn test_delete_query() {
        let query = "delete first from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_none());
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 1);
    }

    #[test]
    fn test_delete_files_query() {
        let query = "delete many from /tmp;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, "/tmp");
        assert!(result.limit.is_none());
    }

    #[test]
    fn test_delete_with_condition() {
        let query = "delete first from . where ext = \"tmp\";";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_some());
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 1);
    }

    #[test]
    fn test_delete_with_limit() {
        let query = "delete many 5 from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 5);
    }

    #[test]
    fn test_delete_recursive() {
        let query = "delete recursive many from .;";

        // Debug: Print the parse tree
        match LSQLParser::parse(Rule::query, query) {
            Ok(pairs) => {
                for pair in pairs {
                    println!("Rule: {:?}", pair.as_rule());
                    println!("Span: {:?}", pair.as_span());
                    println!("Text: {}", pair.as_str());

                    for inner_pair in pair.clone().into_inner() {
                        println!("  Inner Rule: {:?}", inner_pair.as_rule());
                        println!("  Inner Span: {:?}", inner_pair.as_span());
                        println!("  Inner Text: {}", inner_pair.as_str());

                        for inner_inner_pair in inner_pair.clone().into_inner() {
                            println!("    Inner Inner Rule: {:?}", inner_inner_pair.as_rule());
                            println!("    Inner Inner Span: {:?}", inner_inner_pair.as_span());
                            println!("    Inner Inner Text: {}", inner_inner_pair.as_str());
                        }
                    }
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_none());
        assert!(result.is_recursive, "Query should be recursive");
    }

    #[test]
    fn test_delete_with_shorthand_recursive() {
        let query = "delete r first from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_none());
        assert!(
            result.is_recursive,
            "Query should be recursive with shorthand 'r'"
        );
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 1);
    }

    #[test]
    fn test_delete_recursive_with_condition() {
        let query = "delete recursive many 10 from . where ext = \"tmp\";";
        let result = LSQLParser::parse_query(query).unwrap();

        assert_eq!(result.query_type, QueryType::Delete);
        assert!(matches!(result.selection, SelectionType::All));
        assert_eq!(result.path, ".");
        assert!(result.condition.is_some());
        assert!(result.is_recursive, "Query should be recursive");
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 10);
    }

    #[test]
    fn test_order_by_clause() {
        let query = "select type from . where size > 1mb order by modified desc limit 10;";
        let result = LSQLParser::parse_query(query).unwrap();

        // Check selection
        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }

        // Check path
        assert_eq!(result.path, ".");

        // Check condition
        assert!(result.condition.is_some());

        // Check order by
        assert!(!result.order_by.is_empty());
        assert_eq!(result.order_by.len(), 1);
        assert_eq!(result.order_by[0].field, "modified");
        assert_eq!(result.order_by[0].direction, OrderDirection::Descending);

        // Check limit
        assert!(result.limit.is_some());
        assert_eq!(result.limit.unwrap(), 10);
    }
}
