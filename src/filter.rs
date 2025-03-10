// File System Filtering Module
// This module handles filtering file system entries based on conditions

use crate::fs::FSEntry;
use crate::parser::{ComparisonOperator, ConditionNode, LogicalOperator, Value};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use log::{trace, warn};

/// Filters a list of file system entries based on a condition
pub fn filter_entries(entries: Vec<FSEntry>, condition: &Option<ConditionNode>) -> Vec<FSEntry> {
    if condition.is_none() {
        return entries;
    }

    entries
        .into_iter()
        .filter(|entry| entry.matches_condition(condition))
        .collect()
}

/// Extension trait for FSEntry to add condition matching functionality
pub trait ConditionMatcher {
    /// Check if an entry matches the given condition
    fn matches_condition(&self, condition: &Option<ConditionNode>) -> bool;

    /// Evaluate a condition node (leaf or branch)
    fn evaluate_condition_node(&self, node: &ConditionNode) -> bool;

    /// Evaluate a single condition (field comparison)
    fn evaluate_single_condition(
        &self,
        identifier: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool;

    /// Compare a string field with a value
    fn compare_string_field(
        &self,
        field: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool;

    /// Compare the size field with a value
    fn compare_size_field(&self, size: u64, operator: &ComparisonOperator, value: &Value) -> bool;

    /// Compare a date field with a value
    fn compare_date_field(
        &self,
        date: &DateTime<Local>,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool;
}

impl ConditionMatcher for FSEntry {
    fn matches_condition(&self, condition: &Option<ConditionNode>) -> bool {
        match condition {
            Some(node) => self.evaluate_condition_node(node),
            None => true, // No condition means match everything
        }
    }

    fn evaluate_condition_node(&self, node: &ConditionNode) -> bool {
        match node {
            ConditionNode::Leaf(condition) => self.evaluate_single_condition(
                &condition.identifier,
                &condition.operator,
                &condition.value,
            ),
            ConditionNode::Branch {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate_condition_node(left);
                let right_result = self.evaluate_condition_node(right);

                match operator {
                    LogicalOperator::And => left_result && right_result,
                    LogicalOperator::Or => left_result || right_result,
                }
            }
        }
    }

    fn evaluate_single_condition(
        &self,
        identifier: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool {
        trace!(
            "Evaluating condition: {} {:?} {:?}",
            identifier,
            operator,
            value
        );

        match identifier {
            "name" => self.compare_string_field(&self.name, operator, value),
            "path" => self.compare_string_field(&self.path, operator, value),
            "ext" => match &self.extension {
                Some(ext) => self.compare_string_field(ext, operator, value),
                None => false, // No extension means no match
            },
            "size" => self.compare_size_field(self.size, operator, value),
            "modified" => self.compare_date_field(&self.modified, operator, value),
            "created" => self.compare_date_field(&self.created, operator, value),
            "is_hidden" => match value {
                Value::Bool(b) => match operator {
                    ComparisonOperator::Equal => self.is_hidden == *b,
                    ComparisonOperator::NotEqual => self.is_hidden != *b,
                    _ => {
                        warn!("Invalid operator for boolean comparison: {:?}", operator);
                        false
                    }
                },
                _ => {
                    warn!("Invalid value type for boolean comparison: {:?}", value);
                    false
                }
            },
            "is_readonly" => match value {
                Value::Bool(b) => match operator {
                    ComparisonOperator::Equal => (self.permissions.contains("readonly")) == *b,
                    ComparisonOperator::NotEqual => (self.permissions.contains("readonly")) != *b,
                    _ => {
                        warn!("Invalid operator for boolean comparison: {:?}", operator);
                        false
                    }
                },
                _ => {
                    warn!("Invalid value type for boolean comparison: {:?}", value);
                    false
                }
            },
            "type" => match value {
                Value::String(s) => {
                    let type_str = if self.is_dir { "dir" } else { "file" };
                    match operator {
                        ComparisonOperator::Equal => type_str == s,
                        ComparisonOperator::NotEqual => type_str != s,
                        ComparisonOperator::Like => type_str.contains(s),
                        ComparisonOperator::Contains => type_str.contains(s),
                        _ => {
                            warn!("Invalid operator for type comparison: {:?}", operator);
                            false
                        }
                    }
                }
                _ => {
                    warn!("Invalid value type for type comparison: {:?}", value);
                    false
                }
            },
            _ => {
                warn!("Unknown field identifier: {}", identifier);
                false
            }
        }
    }

    fn compare_string_field(
        &self,
        field: &str,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool {
        if let Value::String(s) = value {
            match operator {
                ComparisonOperator::Equal => field == s,
                ComparisonOperator::NotEqual => field != s,
                ComparisonOperator::Like => {
                    // Simple wildcard matching
                    let pattern = s.replace("*", ".*").replace("?", ".");
                    let re = regex::Regex::new(&format!("^{}$", pattern)).unwrap_or_else(|_| {
                        warn!("Invalid regex pattern: {}", pattern);
                        regex::Regex::new("^$").unwrap() // Empty regex that matches nothing
                    });
                    re.is_match(field)
                }
                ComparisonOperator::Contains => field.contains(s),
                _ => {
                    warn!("Invalid operator for string comparison: {:?}", operator);
                    false
                }
            }
        } else {
            warn!(
                "Invalid value type for string comparison: {:?}, expected String",
                value
            );
            false
        }
    }

    fn compare_size_field(&self, size: u64, operator: &ComparisonOperator, value: &Value) -> bool {
        match value {
            Value::Number(n) => {
                let size_f64 = size as f64;
                match operator {
                    ComparisonOperator::Equal => (size_f64 - n).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (size_f64 - n).abs() >= f64::EPSILON,
                    ComparisonOperator::LessThan => size_f64 < *n,
                    ComparisonOperator::LessOrEqual => size_f64 <= *n,
                    ComparisonOperator::GreaterThan => size_f64 > *n,
                    ComparisonOperator::GreaterOrEqual => size_f64 >= *n,
                    _ => {
                        warn!("Invalid operator for number comparison: {:?}", operator);
                        false
                    }
                }
            }
            Value::SizedNumber(n, unit) => {
                let bytes = convert_to_bytes(*n, unit);
                let size_f64 = size as f64;
                match operator {
                    ComparisonOperator::Equal => (size_f64 - bytes).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (size_f64 - bytes).abs() >= f64::EPSILON,
                    ComparisonOperator::LessThan => size_f64 < bytes,
                    ComparisonOperator::LessOrEqual => size_f64 <= bytes,
                    ComparisonOperator::GreaterThan => size_f64 > bytes,
                    ComparisonOperator::GreaterOrEqual => size_f64 >= bytes,
                    _ => {
                        warn!(
                            "Invalid operator for sized number comparison: {:?}",
                            operator
                        );
                        false
                    }
                }
            }
            _ => {
                warn!(
                    "Invalid value type for size comparison: {:?}, expected Number or SizedNumber",
                    value
                );
                false
            }
        }
    }

    fn compare_date_field(
        &self,
        date: &DateTime<Local>,
        operator: &ComparisonOperator,
        value: &Value,
    ) -> bool {
        if let Value::String(s) = value {
            // Parse date string in format YYYY-MM-DD
            if let Ok(naive_date) =
                NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S")
            {
                // Create a DateTime<Local> from the naive date
                let compare_date = match Local.from_local_datetime(&naive_date).single() {
                    Some(dt) => dt,
                    None => {
                        warn!("Failed to convert naive date to local datetime");
                        return false;
                    }
                };

                match operator {
                    ComparisonOperator::Equal => date.date_naive() == compare_date.date_naive(),
                    ComparisonOperator::NotEqual => date.date_naive() != compare_date.date_naive(),
                    ComparisonOperator::LessThan => *date < compare_date,
                    ComparisonOperator::LessOrEqual => *date <= compare_date,
                    ComparisonOperator::GreaterThan => *date > compare_date,
                    ComparisonOperator::GreaterOrEqual => *date >= compare_date,
                    _ => {
                        warn!("Invalid operator for date comparison: {:?}", operator);
                        false
                    }
                }
            } else {
                warn!("Invalid date format: {}, expected YYYY-MM-DD", s);
                false
            }
        } else {
            warn!(
                "Invalid value type for date comparison: {:?}, expected String",
                value
            );
            false
        }
    }
}

/// Convert a sized number to bytes
fn convert_to_bytes(num: f64, unit: &str) -> f64 {
    match unit.to_lowercase().as_str() {
        "b" => num,
        "kb" => num * 1024.0,
        "mb" => num * 1024.0 * 1024.0,
        "gb" => num * 1024.0 * 1024.0 * 1024.0,
        "tb" => num * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => {
            warn!("Unknown size unit: {}, using raw value", unit);
            num
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Condition, Value};
    use chrono::TimeZone;

    // Helper function to create a test FSEntry
    fn create_test_entry() -> FSEntry {
        FSEntry {
            name: "test.txt".to_string(),
            path: "/path/to/test.txt".to_string(),
            size: 1024,
            is_dir: false,
            is_file: true,
            is_hidden: false,
            modified: Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            created: Local.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap(),
            extension: Some("txt".to_string()),
            permissions: "readwrite".to_string(),
        }
    }

    #[test]
    fn test_name_equal_condition() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("test.txt".to_string()),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let condition_not_equal = ConditionNode::Leaf(Condition {
            identifier: "name".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("other.txt".to_string()),
        });

        assert!(!entry.evaluate_condition_node(&condition_not_equal));
    }

    #[test]
    fn test_extension_equal_condition() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "ext".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("txt".to_string()),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let condition_not_equal = ConditionNode::Leaf(Condition {
            identifier: "ext".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("pdf".to_string()),
        });

        assert!(!entry.evaluate_condition_node(&condition_not_equal));
    }

    #[test]
    fn test_size_greater_than_condition() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::Number(500.0),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let condition_not_greater = ConditionNode::Leaf(Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::Number(2000.0),
        });

        assert!(!entry.evaluate_condition_node(&condition_not_greater));
    }

    #[test]
    fn test_size_with_units_condition() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::SizedNumber(0.5, "kb".to_string()),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let condition_not_greater = ConditionNode::Leaf(Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::SizedNumber(2.0, "kb".to_string()),
        });

        assert!(!entry.evaluate_condition_node(&condition_not_greater));
    }

    #[test]
    fn test_date_comparison() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "modified".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::String("2022-01-01".to_string()),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let condition_not_greater = ConditionNode::Leaf(Condition {
            identifier: "modified".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::String("2023-02-01".to_string()),
        });

        assert!(!entry.evaluate_condition_node(&condition_not_greater));
    }

    #[test]
    fn test_is_hidden_condition() {
        let entry = create_test_entry();
        let condition = ConditionNode::Leaf(Condition {
            identifier: "is_hidden".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::Bool(false),
        });

        assert!(entry.evaluate_condition_node(&condition));

        let mut hidden_entry = entry.clone();
        hidden_entry.is_hidden = true;

        assert!(!hidden_entry.evaluate_condition_node(&condition));
    }

    #[test]
    fn test_complex_condition() {
        let entry = create_test_entry();

        // (ext = "txt" AND size > 500) OR name CONTAINS "test"
        let condition = ConditionNode::Branch {
            left: Box::new(ConditionNode::Branch {
                left: Box::new(ConditionNode::Leaf(Condition {
                    identifier: "ext".to_string(),
                    operator: ComparisonOperator::Equal,
                    value: Value::String("txt".to_string()),
                })),
                operator: LogicalOperator::And,
                right: Box::new(ConditionNode::Leaf(Condition {
                    identifier: "size".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: Value::Number(500.0),
                })),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ConditionNode::Leaf(Condition {
                identifier: "name".to_string(),
                operator: ComparisonOperator::Contains,
                value: Value::String("test".to_string()),
            })),
        };

        assert!(entry.evaluate_condition_node(&condition));
    }

    #[test]
    fn test_filter_entries() {
        let entries = vec![
            create_test_entry(),
            FSEntry {
                name: "image.png".to_string(),
                path: "/path/to/image.png".to_string(),
                size: 2048,
                is_dir: false,
                is_file: true,
                is_hidden: false,
                modified: Local.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap(),
                created: Local.with_ymd_and_hms(2022, 2, 1, 0, 0, 0).unwrap(),
                extension: Some("png".to_string()),
                permissions: "readwrite".to_string(),
            },
            FSEntry {
                name: "docs".to_string(),
                path: "/path/to/docs".to_string(),
                size: 4096,
                is_dir: true,
                is_file: false,
                is_hidden: false,
                modified: Local.with_ymd_and_hms(2023, 3, 1, 0, 0, 0).unwrap(),
                created: Local.with_ymd_and_hms(2022, 3, 1, 0, 0, 0).unwrap(),
                extension: None,
                permissions: "readwrite".to_string(),
            },
        ];

        // Filter for txt files
        let condition = Some(ConditionNode::Leaf(Condition {
            identifier: "ext".to_string(),
            operator: ComparisonOperator::Equal,
            value: Value::String("txt".to_string()),
        }));

        let filtered = filter_entries(entries.clone(), &condition);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "test.txt");

        // Filter for files larger than 1.5KB
        let condition = Some(ConditionNode::Leaf(Condition {
            identifier: "size".to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Value::SizedNumber(1.5, "kb".to_string()),
        }));

        let filtered = filter_entries(entries.clone(), &condition);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "image.png");
        assert_eq!(filtered[1].name, "docs");
    }
}
