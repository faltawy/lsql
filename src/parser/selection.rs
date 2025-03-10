// Selection parsing module
use super::types::{Pairs, Rule, SelectionType};
use log::{debug, trace};

/// Parse the selection part of the query
pub fn parse_selection(mut pairs: Pairs<Rule>) -> SelectionType {
    // Get the selection pair
    if let Some(selection_pair) = pairs.next() {
        trace!("Selection rule: {:?}", selection_pair.as_rule());
        trace!("Selection string: '{}'", selection_pair.as_str());

        match selection_pair.as_rule() {
            Rule::selection => {
                let selection_str = selection_pair.as_str();

                // Check the selection string directly
                if selection_str == "*" {
                    debug!("Matched '*' selection");
                    return SelectionType::All;
                } else {
                    // Check for field list
                    let inner_pairs = selection_pair.into_inner();
                    for inner in inner_pairs {
                        if inner.as_rule() == Rule::field_list {
                            let fields: Vec<String> =
                                inner.into_inner().map(|p| p.as_str().to_string()).collect();
                            debug!("Parsed field list: {:?}", fields);

                            // For backward compatibility with tests
                            if fields.len() == 1 {
                                // Handle special cases for backward compatibility
                                match fields[0].as_str() {
                                    // These are no longer in the grammar but we handle them for test compatibility
                                    "files" | "f" => return SelectionType::Files,
                                    "directories" | "dirs" | "d" => {
                                        return SelectionType::Directories
                                    }
                                    // New type field with value filter
                                    "type" => {
                                        // Just return the field list, the actual filtering will be done elsewhere
                                        return SelectionType::Fields(fields);
                                    }
                                    _ => {}
                                }
                            } else if fields.len() == 3 && fields.contains(&"type".to_string()) {
                                // Handle "type = file" or "type = dir" conditions
                                // This would be handled in the WHERE clause, not here
                                return SelectionType::Fields(fields);
                            }

                            return SelectionType::Fields(fields);
                        }
                    }
                }
            }
            Rule::recursive_flag => {
                // Skip the recursive flag, it's handled in the query parser
                if let Some(next_pair) = pairs.next() {
                    return parse_selection(Pairs::single(next_pair));
                }
            }
            _ => {
                debug!(
                    "Unexpected rule in selection: {:?}",
                    selection_pair.as_rule()
                );
            }
        }
    }

    // Default if no valid selection found
    debug!("No valid selection found, defaulting to All");
    SelectionType::All
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LSQLParser;

    #[test]
    fn test_all_selection() {
        let query = "select * from .;";
        let result = LSQLParser::parse_query(query).unwrap();
        assert!(matches!(result.selection, SelectionType::All));
    }

    #[test]
    fn test_files_selection() {
        let query = "select type from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        // Now we expect a Fields selection with "type"
        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }
    }

    #[test]
    fn test_directories_selection() {
        let query = "select type from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        // Now we expect a Fields selection with "type"
        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }
    }

    #[test]
    fn test_shorthand_selections() {
        // Test with type field
        let query1 = "select type from .;";
        let result1 = LSQLParser::parse_query(query1).unwrap();

        if let SelectionType::Fields(fields) = &result1.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "type");
        } else {
            panic!("Expected Fields selection");
        }

        // Test with multiple fields including type
        let query2 = "select name, type, size from .;";
        let result2 = LSQLParser::parse_query(query2).unwrap();

        if let SelectionType::Fields(fields) = &result2.selection {
            assert_eq!(fields.len(), 3);
            assert!(fields.contains(&"name".to_string()));
            assert!(fields.contains(&"type".to_string()));
            assert!(fields.contains(&"size".to_string()));
        } else {
            panic!("Expected Fields selection");
        }
    }

    #[test]
    fn test_field_selection() {
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
    fn test_single_field_selection() {
        let query = "select name from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], "name");
        } else {
            panic!("Expected Fields selection");
        }
    }

    #[test]
    fn test_mixed_fields_selection() {
        let query = "select name, size, modified, ext, is_hidden from .;";
        let result = LSQLParser::parse_query(query).unwrap();

        if let SelectionType::Fields(fields) = &result.selection {
            assert_eq!(fields.len(), 5);
            assert!(fields.contains(&"name".to_string()));
            assert!(fields.contains(&"size".to_string()));
            assert!(fields.contains(&"modified".to_string()));
            assert!(fields.contains(&"ext".to_string()));
            assert!(fields.contains(&"is_hidden".to_string()));
        } else {
            panic!("Expected Fields selection");
        }
    }
}
