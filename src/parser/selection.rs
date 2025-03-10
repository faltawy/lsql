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
                match selection_str {
                    "*" => {
                        debug!("Matched '*' selection");
                        return SelectionType::All;
                    }
                    "files" => {
                        debug!("Matched 'files' selection");
                        return SelectionType::Files;
                    }
                    "f" => {
                        debug!("Matched 'f' selection (shorthand for files)");
                        return SelectionType::Files;
                    }
                    "directories" => {
                        debug!("Matched 'directories' selection");
                        return SelectionType::Directories;
                    }
                    "dirs" => {
                        debug!("Matched 'dirs' selection (shorthand for directories)");
                        return SelectionType::Directories;
                    }
                    "d" => {
                        debug!("Matched 'd' selection (shorthand for directories)");
                        return SelectionType::Directories;
                    }
                    _ => {
                        // Check for field list
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
        let query = "select files from .;";
        let result = LSQLParser::parse_query(query).unwrap();
        assert!(matches!(result.selection, SelectionType::Files));
    }

    #[test]
    fn test_directories_selection() {
        let query = "select directories from .;";
        let result = LSQLParser::parse_query(query).unwrap();
        assert!(matches!(result.selection, SelectionType::Directories));
    }

    #[test]
    fn test_shorthand_selections() {
        // Test directory shorthand "dirs"
        let query1 = "select dirs from .;";
        let result1 = LSQLParser::parse_query(query1).unwrap();
        assert!(matches!(result1.selection, SelectionType::Directories));

        // Test file shorthand "f"
        let query2 = "select f from .;";
        let result2 = LSQLParser::parse_query(query2).unwrap();
        assert!(matches!(result2.selection, SelectionType::Files));

        // Test directory shorthand "d"
        let query3 = "select d from .;";
        let result3 = LSQLParser::parse_query(query3).unwrap();
        assert!(matches!(result3.selection, SelectionType::Directories));
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
