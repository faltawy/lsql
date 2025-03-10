// Types and enums for the SQL-like syntax
use pest_derive::Parser;

// Represents the type of query (SELECT or DELETE)
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    /// Select entries (read-only)
    Select,
    /// Delete entries
    Delete,
}

// Represents what should be selected (files, directories, or both)
#[derive(Debug, Clone)]
pub enum SelectionType {
    /// Select all entries (both files and directories)
    All,
    /// Select only files
    Files,
    /// Select only directories
    Directories,
    /// Select specific fields from entries
    Fields(Vec<String>),
}

// Represents a comparison operation in the WHERE clause
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    /// Equality comparison (=)
    Equal,
    /// Inequality comparison (!=)
    NotEqual,
    /// Less than comparison (<)
    LessThan,
    /// Less than or equal comparison (<=)
    LessOrEqual,
    /// Greater than comparison (>)
    GreaterThan,
    /// Greater than or equal comparison (>=)
    GreaterOrEqual,
    /// Pattern matching using wildcards (like)
    Like,
    /// Substring matching (contains)
    Contains,
}

// Represents a value in a comparison
#[derive(Debug, Clone)]
pub enum Value {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Numeric value with a unit (e.g., "10mb")
    SizedNumber(f64, String),
    /// Boolean value
    Bool(bool),
}

// Represents a condition in the WHERE clause
#[derive(Debug, Clone)]
pub struct Condition {
    /// Field identifier being compared
    pub identifier: String,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Value being compared against
    pub value: Value,
}

// Represents a logical operation between conditions
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

// Represents a condition node in the condition tree
#[derive(Debug, Clone)]
pub enum ConditionNode {
    /// A single comparison condition
    Leaf(Condition),
    /// A branch with two subconditions connected by a logical operator
    Branch {
        left: Box<ConditionNode>,
        operator: LogicalOperator,
        right: Box<ConditionNode>,
    },
}

// Re-exported from the pest crate for use in other modules
pub use pest::iterators::{Pair, Pairs};

// Import the grammar rules enum from the pest parser
#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[grammar = "parser/lsql.pest"]
pub struct LSQLParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_type_variants() {
        // Test that each variant can be created
        let all = SelectionType::All;
        let files = SelectionType::Files;
        let dirs = SelectionType::Directories;
        let fields = SelectionType::Fields(vec!["name".to_string(), "size".to_string()]);

        assert!(matches!(all, SelectionType::All));
        assert!(matches!(files, SelectionType::Files));
        assert!(matches!(dirs, SelectionType::Directories));
        if let SelectionType::Fields(f) = fields {
            assert_eq!(f.len(), 2);
            assert!(f.contains(&"name".to_string()));
            assert!(f.contains(&"size".to_string()));
        } else {
            panic!("Expected Fields variant");
        }
    }

    #[test]
    fn test_comparison_operators() {
        // Test all comparison operators
        let ops = vec![
            ComparisonOperator::Equal,
            ComparisonOperator::NotEqual,
            ComparisonOperator::LessThan,
            ComparisonOperator::LessOrEqual,
            ComparisonOperator::GreaterThan,
            ComparisonOperator::GreaterOrEqual,
            ComparisonOperator::Like,
            ComparisonOperator::Contains,
        ];

        // Verify we have 8 operators
        assert_eq!(ops.len(), 8);

        // Ensure each variant is distinct
        for (i, op1) in ops.iter().enumerate() {
            for (j, op2) in ops.iter().enumerate() {
                if i == j {
                    assert!(matches!(op1, op2));
                } else {
                    assert!(!std::mem::discriminant(op1).eq(&std::mem::discriminant(op2)));
                }
            }
        }
    }

    #[test]
    fn test_value_variants() {
        // Test string value
        let string_val = Value::String("test".to_string());
        if let Value::String(s) = string_val {
            assert_eq!(s, "test");
        } else {
            panic!("Expected String variant");
        }

        // Test number value
        let num_val = Value::Number(42.5);
        if let Value::Number(n) = num_val {
            assert_eq!(n, 42.5);
        } else {
            panic!("Expected Number variant");
        }

        // Test sized number value
        let sized_val = Value::SizedNumber(10.0, "mb".to_string());
        if let Value::SizedNumber(n, unit) = sized_val {
            assert_eq!(n, 10.0);
            assert_eq!(unit, "mb");
        } else {
            panic!("Expected SizedNumber variant");
        }

        // Test boolean value
        let bool_val = Value::Bool(true);
        if let Value::Bool(b) = bool_val {
            assert!(b);
        } else {
            panic!("Expected Bool variant");
        }
    }

    #[test]
    fn test_condition_node_structure() {
        // Create a simple leaf condition
        let condition = Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("test.txt".to_string()),
        };
        let leaf = ConditionNode::Leaf(condition);

        // Create another simple condition
        let condition2 = Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::Number(1000.0),
        };
        let leaf2 = ConditionNode::Leaf(condition2);

        // Create a branch condition
        let branch = ConditionNode::Branch {
            left: Box::new(leaf),
            operator: LogicalOperator::And,
            right: Box::new(leaf2),
        };

        // Test that we can extract components correctly
        if let ConditionNode::Branch {
            left,
            operator,
            right,
        } = branch
        {
            // Verify operator
            assert!(matches!(operator, LogicalOperator::And));

            // Check left condition
            if let ConditionNode::Leaf(cond) = *left {
                assert_eq!(cond.identifier, "name");
                assert!(matches!(cond.operator, ComparisonOperator::Equal));
                if let Value::String(s) = cond.value {
                    assert_eq!(s, "test.txt");
                } else {
                    panic!("Expected String value");
                }
            } else {
                panic!("Expected Leaf node");
            }

            // Check right condition
            if let ConditionNode::Leaf(cond) = *right {
                assert_eq!(cond.identifier, "size");
                assert!(matches!(cond.operator, ComparisonOperator::GreaterThan));
                if let Value::Number(n) = cond.value {
                    assert_eq!(n, 1000.0);
                } else {
                    panic!("Expected Number value");
                }
            } else {
                panic!("Expected Leaf node");
            }
        } else {
            panic!("Expected Branch node");
        }
    }
}
