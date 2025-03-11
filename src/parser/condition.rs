// Condition parsing module
use super::types::{ComparisonOperator, Condition, ConditionNode, LogicalOperator, Pairs, Rule};
use super::value;
use log::{debug, trace, warn};

/// Parse the condition part of the query
pub fn parse_condition(mut pairs: Pairs<Rule>) -> ConditionNode {
    if let Some(condition_pair) = pairs.next() {
        debug!("Parsing condition: {}", condition_pair.as_str());
        parse_condition_node(condition_pair.into_inner())
    } else {
        // This should not happen with valid input, but provide a fallback
        warn!("No condition found in WHERE clause");
        panic!("No condition found in WHERE clause")
    }
}

/// Recursively parse condition nodes
fn parse_condition_node(mut pairs: Pairs<Rule>) -> ConditionNode {
    if let Some(first_pair) = pairs.next() {
        trace!("Parsing condition node: {}", first_pair.as_str());
        let first_condition = parse_primary_condition(first_pair.into_inner());

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
                let next_condition = parse_primary_condition(next_pair.into_inner());

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

/// Parse a primary condition (either a comparison or a nested condition)
fn parse_primary_condition(mut pairs: Pairs<Rule>) -> ConditionNode {
    if let Some(pair) = pairs.next() {
        trace!("Parsing primary condition: {}", pair.as_str());

        match pair.as_rule() {
            Rule::condition => parse_condition_node(pair.into_inner()),
            Rule::comparison => {
                let mut inner_pairs = pair.into_inner();

                if let (Some(id_pair), Some(op_pair), Some(val_pair)) =
                    (inner_pairs.next(), inner_pairs.next(), inner_pairs.next())
                {
                    let identifier = id_pair.as_str().to_string();
                    let op_rule = op_pair.as_rule();
                    let op_str = op_pair.as_str().to_string();
                    debug!("Operator rule: {:?}, str: {}", op_rule, op_str);
                    let operator = match op_pair.into_inner().next().unwrap().as_rule() {
                        Rule::EQUAL => ComparisonOperator::Equal,
                        Rule::NOT_EQUAL => ComparisonOperator::NotEqual,
                        Rule::LESS => ComparisonOperator::LessThan,
                        Rule::LESS_OR_EQUAL => ComparisonOperator::LessOrEqual,
                        Rule::GREATER => ComparisonOperator::GreaterThan,
                        Rule::GREATER_OR_EQUAL => ComparisonOperator::GreaterOrEqual,
                        Rule::LIKE => ComparisonOperator::Like,
                        Rule::CONTAINS => ComparisonOperator::Contains,
                        _ => {
                            warn!(
                                "Unknown comparison operator: {}, defaulting to EQUAL",
                                op_str
                            );
                            ComparisonOperator::Equal // Default
                        }
                    };

                    let value = value::parse_value(val_pair);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{LSQLParser, Value};

    #[test]
    fn test_equal_condition() {
        let query = "select * from . where ext = \"png\";";
        let result = LSQLParser::parse_query(query).unwrap();

        assert!(result.condition.is_some());
        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition.identifier, "ext");
        assert!(matches!(condition.operator, ComparisonOperator::Equal));

        // Check value type
        if let Value::String(s) = &condition.value {
            assert_eq!(s, "png");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_not_equal_condition() {
        let query = "select * from . where size != 0;";
        let result = LSQLParser::parse_query(query).unwrap();

        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition.identifier, "size");
        assert!(matches!(condition.operator, ComparisonOperator::NotEqual));

        // Check value
        if let Value::Number(n) = condition.value {
            assert_eq!(n, 0.0);
        } else {
            panic!("Expected Number value");
        }
    }

    #[test]
    fn test_greater_than_condition() {
        let query = "select * from . where size > 1024;";
        let result = LSQLParser::parse_query(query).unwrap();

        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition.identifier, "size");
        assert!(matches!(
            condition.operator,
            ComparisonOperator::GreaterThan
        ));

        // Check value
        if let Value::Number(n) = condition.value {
            assert_eq!(n, 1024.0);
        } else {
            panic!("Expected Number value");
        }
    }

    #[test]
    fn test_less_than_condition() {
        let query = "select * from . where modified < \"2023-01-01\";";
        let result = LSQLParser::parse_query(query).unwrap();

        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition.identifier, "modified");
        assert!(matches!(condition.operator, ComparisonOperator::LessThan));

        // Check value
        if let Value::String(s) = &condition.value {
            assert_eq!(s, "2023-01-01");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_less_equal_greater_equal_operators() {
        // Test less than or equal
        let query1 = "select * from . where size <= 2048;";
        let result1 = LSQLParser::parse_query(query1).unwrap();

        let condition1 = match &result1.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition1.identifier, "size");
        assert!(matches!(
            condition1.operator,
            ComparisonOperator::LessOrEqual
        ));

        // Test greater than or equal
        let query2 = "select * from . where size >= 4096;";
        let result2 = LSQLParser::parse_query(query2).unwrap();

        let condition2 = match &result2.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition"),
        };

        assert_eq!(condition2.identifier, "size");
        assert!(matches!(
            condition2.operator,
            ComparisonOperator::GreaterOrEqual
        ));
    }

    #[test]
    fn test_and_condition() {
        let query = "select * from . where size > 1mb and ext = \"pdf\";";
        let result = LSQLParser::parse_query(query).unwrap();

        match &result.condition {
            Some(ConditionNode::Branch {
                left,
                operator,
                right,
            }) => {
                // Check operator
                assert!(matches!(operator, LogicalOperator::And));

                // Check left side (size > 1mb)
                if let ConditionNode::Leaf(cond) = &**left {
                    assert_eq!(cond.identifier, "size");
                    assert!(matches!(cond.operator, ComparisonOperator::GreaterThan));
                    if let Value::SizedNumber(n, unit) = &cond.value {
                        assert_eq!(*n, 1.0);
                        assert_eq!(unit, "mb");
                    } else {
                        panic!("Expected SizedNumber value");
                    }
                } else {
                    panic!("Expected Leaf node for left side");
                }

                // Check right side (ext = "pdf")
                if let ConditionNode::Leaf(cond) = &**right {
                    assert_eq!(cond.identifier, "ext");
                    assert!(matches!(cond.operator, ComparisonOperator::Equal));
                    if let Value::String(s) = &cond.value {
                        assert_eq!(s, "pdf");
                    } else {
                        panic!("Expected String value");
                    }
                } else {
                    panic!("Expected Leaf node for right side");
                }
            }
            _ => panic!("Expected a branch condition with AND operator"),
        }
    }

    #[test]
    fn test_or_condition() {
        let query = "select * from . where ext = \"jpg\" or ext = \"png\";";
        let result = LSQLParser::parse_query(query).unwrap();

        match &result.condition {
            Some(ConditionNode::Branch { operator, .. }) => {
                assert!(matches!(operator, LogicalOperator::Or));
            }
            _ => panic!("Expected a branch condition with OR operator"),
        }
    }

    #[test]
    fn test_like_operator() {
        let query = "select * from . where name like \"*.rs\";";
        let result = LSQLParser::parse_query(query).unwrap();

        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition for LIKE"),
        };

        assert_eq!(condition.identifier, "name");
        assert!(matches!(condition.operator, ComparisonOperator::Like));

        // Check value
        if let Value::String(s) = &condition.value {
            assert_eq!(s, "*.rs");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_contains_operator() {
        let query = "select * from . where name contains \"main\";";
        let result = LSQLParser::parse_query(query).unwrap();

        let condition = match &result.condition {
            Some(ConditionNode::Leaf(c)) => c,
            _ => panic!("Expected a leaf condition for CONTAINS"),
        };

        assert_eq!(condition.identifier, "name");
        assert!(matches!(condition.operator, ComparisonOperator::Contains));

        // Check value
        if let Value::String(s) = &condition.value {
            assert_eq!(s, "main");
        } else {
            panic!("Expected String value");
        }
    }
}
