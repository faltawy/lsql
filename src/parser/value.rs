// Value parsing module
use super::types::{Pair, Rule, Value};
use log::{trace, warn};

/// Parse a value (string, number, or boolean)
pub fn parse_value(pair: Pair<Rule>) -> Value {
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
            if let Some(unit_start) = s.find(|c: char| !c.is_ascii_digit() && c != '.') {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LSQLParser;

    #[test]
    fn test_parse_string_value() {
        let query = "select * from . where name = \"test.txt\";";
        let result = LSQLParser::parse_query(query).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result.condition {
            if let Value::String(s) = &condition.value {
                assert_eq!(s, "test.txt");
            } else {
                panic!("Expected String value");
            }
        } else {
            panic!("Expected leaf condition node");
        }
    }

    #[test]
    fn test_parse_number_value() {
        let query = "select * from . where size = 1024;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result.condition {
            if let Value::Number(n) = condition.value {
                assert_eq!(n, 1024.0);
            } else {
                panic!("Expected Number value");
            }
        } else {
            panic!("Expected leaf condition node");
        }
    }

    #[test]
    fn test_parse_sized_number_value() {
        // Test with megabytes
        let query1 = "select * from . where size > 5mb;";
        let result1 = LSQLParser::parse_query(query1).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result1.condition {
            if let Value::SizedNumber(n, unit) = &condition.value {
                assert_eq!(*n, 5.0);
                assert_eq!(unit, "mb");
            } else {
                panic!("Expected SizedNumber value");
            }
        } else {
            panic!("Expected leaf condition node");
        }

        // Test with kilobytes
        let query2 = "select * from . where size > 500kb;";
        let result2 = LSQLParser::parse_query(query2).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result2.condition {
            if let Value::SizedNumber(n, unit) = &condition.value {
                assert_eq!(*n, 500.0);
                assert_eq!(unit, "kb");
            } else {
                panic!("Expected SizedNumber value");
            }
        } else {
            panic!("Expected leaf condition node");
        }

        // Test with gigabytes
        let query3 = "select * from . where size > 1.5gb;";
        let result3 = LSQLParser::parse_query(query3).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result3.condition {
            if let Value::SizedNumber(n, unit) = &condition.value {
                assert_eq!(*n, 1.5);
                assert_eq!(unit, "gb");
            } else {
                panic!("Expected SizedNumber value");
            }
        } else {
            panic!("Expected leaf condition node");
        }
    }

    #[test]
    fn test_parse_bool_value() {
        // Test true
        let query1 = "select * from . where is_hidden = true;";
        let result1 = LSQLParser::parse_query(query1).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result1.condition {
            if let Value::Bool(b) = condition.value {
                assert!(b);
            } else {
                panic!("Expected Bool value");
            }
        } else {
            panic!("Expected leaf condition node");
        }

        // Test false
        let query2 = "select * from . where is_hidden = false;";
        let result2 = LSQLParser::parse_query(query2).unwrap();

        if let Some(crate::parser::ConditionNode::Leaf(condition)) = &result2.condition {
            if let Value::Bool(b) = condition.value {
                assert!(!b);
            } else {
                panic!("Expected Bool value");
            }
        } else {
            panic!("Expected leaf condition node");
        }
    }

    #[test]
    fn test_value_in_complex_condition() {
        let query =
            "select * from . where size > 5mb and (ext = \"pdf\" or name contains \"report\");";
        let result = LSQLParser::parse_query(query).unwrap();

        // Extract the size condition from the left side of the AND
        if let Some(crate::parser::ConditionNode::Branch { left, .. }) = &result.condition {
            if let crate::parser::ConditionNode::Leaf(condition) = &**left {
                if let Value::SizedNumber(n, unit) = &condition.value {
                    assert_eq!(*n, 5.0);
                    assert_eq!(unit, "mb");
                } else {
                    panic!("Expected SizedNumber value");
                }
            } else {
                panic!("Expected leaf condition node");
            }
        } else {
            panic!("Expected branch condition node");
        }
    }
}
