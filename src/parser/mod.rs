// Parser module for LSQL
// This module is responsible for parsing SQL-like queries for file system operations

use log::{debug, trace, warn};
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/lsql.pest"]
pub struct LSQLParser;

// Represents what should be selected (files, directories, or both)
#[derive(Debug, Clone)]
pub enum SelectionType {
    All,
    Files,
    Directories,
    Fields(Vec<String>),
}

// Represents a comparison operation in the WHERE clause
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Like,
    Contains,
}

// Represents a value in a comparison
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    SizedNumber(f64, String), // For values like "10mb"
    Bool(bool),
}

// Represents a condition in the WHERE clause
#[derive(Debug, Clone)]
pub struct Condition {
    pub identifier: String,
    pub operator: ComparisonOperator,
    pub value: Value,
}

// Represents a logical operation between conditions
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

// Represents a condition node in the condition tree
#[derive(Debug, Clone)]
pub enum ConditionNode {
    Leaf(Condition),
    Branch {
        left: Box<ConditionNode>,
        operator: LogicalOperator,
        right: Box<ConditionNode>,
    },
}

// Represents a parsed query
#[derive(Debug, Clone)]
pub struct Query {
    pub selection: SelectionType,
    pub path: String,
    pub condition: Option<ConditionNode>,
}

impl LSQLParser {
    // Parse a query string into a Query struct
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

        // Process each part of the query
        for pair in pairs {
            trace!("Processing rule: {:?}", pair.as_rule());

            match pair.as_rule() {
                Rule::select_clause => {
                    debug!("Found select_clause: {}", pair.as_str());
                    selection = Self::parse_selection(pair.into_inner());
                }
                Rule::from_clause => {
                    debug!("Found from_clause: {}", pair.as_str());
                    path = Self::parse_path(pair.into_inner());
                }
                Rule::where_clause => {
                    debug!("Found where_clause: {}", pair.as_str());
                    condition = Some(Self::parse_condition(pair.into_inner()));
                }
                _ => {
                    trace!("Found unknown rule: {}", pair.as_str());
                }
            }
        }

        debug!(
            "Parsed query: selection={:?}, path={}, condition={}",
            selection,
            path,
            if condition.is_some() {
                "present"
            } else {
                "none"
            }
        );

        Ok(Query {
            selection,
            path,
            condition,
        })
    }

    // Parse the selection part of the query
    fn parse_selection(mut pairs: Pairs<Rule>) -> SelectionType {
        // Get the selection pair
        if let Some(selection_pair) = pairs.next() {
            let selection_str = selection_pair.as_str();
            trace!("Selection string: '{}'", selection_str);

            // Check for direct matches in the selection string
            if selection_str.contains("files") || selection_str.contains("f") {
                debug!("Matched 'files' selection");
                return SelectionType::Files;
            } else if selection_str.contains("directories")
                || selection_str.contains("dirs")
                || selection_str.contains("d")
            {
                debug!("Matched 'directories' selection");
                return SelectionType::Directories;
            } else if selection_str.contains("*") {
                debug!("Matched '*' selection");
                return SelectionType::All;
            }

            // Try to parse as field list
            let inner_pairs = selection_pair.into_inner();
            for inner in inner_pairs {
                if inner.as_rule() == Rule::field_list {
                    let fields: Vec<String> =
                        inner.into_inner().map(|p| p.as_str().to_string()).collect();
                    debug!("Parsed field list: {:?}", fields);
                    return SelectionType::Fields(fields);
                }
            }
        }

        // Default if no valid selection found
        debug!("No valid selection found, defaulting to All");
        SelectionType::All
    }

    // Parse the path part of the query
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

    // Parse the condition part of the query
    fn parse_condition(mut pairs: Pairs<Rule>) -> ConditionNode {
        if let Some(condition_pair) = pairs.next() {
            debug!("Parsing condition: {}", condition_pair.as_str());
            Self::parse_condition_node(condition_pair.into_inner())
        } else {
            // This should not happen with valid input, but provide a fallback
            warn!("No condition found in WHERE clause");
            panic!("No condition found in WHERE clause")
        }
    }

    // Recursively parse condition nodes
    fn parse_condition_node(mut pairs: Pairs<Rule>) -> ConditionNode {
        if let Some(first_pair) = pairs.next() {
            trace!("Parsing condition node: {}", first_pair.as_str());
            let first_condition = Self::parse_primary_condition(first_pair.into_inner());

            // If there are more conditions, build a tree
            let mut current_node = first_condition;

            while let Some(op_pair) = pairs.next() {
                if let Some(next_pair) = pairs.next() {
                    let operator = match op_pair.as_str() {
                        "and" => LogicalOperator::And,
                        "or" => LogicalOperator::Or,
                        _ => {
                            warn!(
                                "Unknown logical operator: {}, defaulting to AND",
                                op_pair.as_str()
                            );
                            LogicalOperator::And // Default
                        }
                    };

                    trace!("Found logical operator: {:?}", operator);
                    let next_condition = Self::parse_primary_condition(next_pair.into_inner());

                    current_node = ConditionNode::Branch {
                        left: Box::new(current_node),
                        operator,
                        right: Box::new(next_condition),
                    };
                }
            }

            current_node
        } else {
            // This should not happen with valid input, but provide a fallback
            warn!("Empty condition node");
            panic!("Empty condition node")
        }
    }

    // Parse a primary condition (either a comparison or a nested condition)
    fn parse_primary_condition(mut pairs: Pairs<Rule>) -> ConditionNode {
        if let Some(pair) = pairs.next() {
            trace!("Parsing primary condition: {}", pair.as_str());

            match pair.as_rule() {
                Rule::condition => Self::parse_condition_node(pair.into_inner()),
                Rule::comparison => {
                    let mut inner_pairs = pair.into_inner();

                    if let (Some(id_pair), Some(op_pair), Some(val_pair)) =
                        (inner_pairs.next(), inner_pairs.next(), inner_pairs.next())
                    {
                        let identifier = id_pair.as_str().to_string();
                        let op_str = op_pair.as_str();

                        let operator = match op_str {
                            "=" => ComparisonOperator::Equal,
                            "!=" => ComparisonOperator::NotEqual,
                            "<" => ComparisonOperator::LessThan,
                            "<=" => ComparisonOperator::LessOrEqual,
                            ">" => ComparisonOperator::GreaterThan,
                            ">=" => ComparisonOperator::GreaterOrEqual,
                            "like" => ComparisonOperator::Like,
                            "contains" => ComparisonOperator::Contains,
                            _ => {
                                warn!(
                                    "Unknown comparison operator: {}, defaulting to EQUAL",
                                    op_str
                                );
                                ComparisonOperator::Equal // Default
                            }
                        };

                        let value = Self::parse_value(val_pair);

                        debug!(
                            "Parsed comparison: {} {:?} {:?}",
                            identifier, operator, value
                        );

                        ConditionNode::Leaf(Condition {
                            identifier,
                            operator,
                            value,
                        })
                    } else {
                        warn!("Invalid comparison: missing components");
                        panic!("Invalid comparison: missing components")
                    }
                }
                _ => {
                    warn!("Unexpected rule in primary condition: {:?}", pair.as_rule());
                    panic!("Unexpected rule in primary condition")
                }
            }
        } else {
            warn!("Empty primary condition");
            panic!("Empty primary condition")
        }
    }

    // Parse a value (string, number, or boolean)
    fn parse_value(pair: pest::iterators::Pair<Rule>) -> Value {
        trace!("Parsing value: {}", pair.as_str());

        match pair.as_rule() {
            Rule::string => {
                let s = pair.as_str();
                // Remove quotes
                let value = Value::String(s[1..s.len() - 1].to_string());
                trace!("Parsed string value: {:?}", value);
                value
            }
            Rule::number => {
                let s = pair.as_str();
                if let Some(unit_start) = s.find(|c: char| !c.is_digit(10) && c != '.') {
                    let (num_str, unit) = s.split_at(unit_start);
                    if let Ok(num) = num_str.parse::<f64>() {
                        let value = Value::SizedNumber(num, unit.to_string());
                        trace!("Parsed sized number value: {:?}", value);
                        value
                    } else {
                        warn!("Failed to parse number: {}, defaulting to 0.0", num_str);
                        Value::Number(0.0) // Default on error
                    }
                } else if let Ok(num) = s.parse::<f64>() {
                    let value = Value::Number(num);
                    trace!("Parsed number value: {:?}", value);
                    value
                } else {
                    warn!("Failed to parse number: {}, defaulting to 0.0", s);
                    Value::Number(0.0) // Default on error
                }
            }
            Rule::bool => {
                let value = Value::Bool(pair.as_str() == "true");
                trace!("Parsed boolean value: {:?}", value);
                value
            }
            _ => {
                warn!(
                    "Unknown value type: {:?}, using empty string",
                    pair.as_rule()
                );
                Value::String("".to_string()) // Default
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let query = "select * from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        match result.selection {
            SelectionType::All => (),
            _ => panic!("Expected All selection"),
        }

        assert_eq!(result.path, ".");
        assert!(result.condition.is_none());
    }

    #[test]
    fn test_files_only_query() {
        let query = "select files from /tmp;";
        let result = LSQLParser::parse_query(query).unwrap();

        match result.selection {
            SelectionType::Files => (),
            _ => panic!("Expected Files selection"),
        }

        assert_eq!(result.path, "/tmp");
    }

    #[test]
    fn test_with_condition() {
        let query = "select * from . where ext = \"png\";";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(result.condition.is_some());

        if let Some(ConditionNode::Leaf(condition)) = result.condition {
            assert_eq!(condition.identifier, "ext");
            match condition.operator {
                ComparisonOperator::Equal => (),
                _ => panic!("Expected Equal operator"),
            }
            match condition.value {
                Value::String(s) if s == "png" => (),
                _ => panic!("Expected string value 'png'"),
            }
        } else {
            panic!("Expected leaf condition node");
        }
    }
}
