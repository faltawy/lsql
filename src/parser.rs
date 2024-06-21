use nom::{
    branch::alt, bytes::complete::{tag, tag_no_case, take_while, take_while1}, character::complete::{char, multispace0}, combinator::{map, opt}, multi::separated_list0, sequence::{delimited, preceded, tuple}, IResult, Parser
};

#[derive(Debug, PartialEq)]
pub enum WhereClause {
    Equal(String, String),
    NotEqual(String, String),
    LessThan(String, String),
    LessThanOrEqual(String, String),
    GreaterThan(String, String),
    GreaterThanOrEqual(String, String),
    UnknownOperator(String, String),
}


#[derive(Debug, PartialEq)]
pub enum WhereType<'a> {
    Conditions(Vec<(&'a str, &'a str, &'a str)>),
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Select {
        columns: Vec<String>,
        where_clause: Option<Vec<WhereClause>>,
        order_by: Option<Vec<String>>,
    },
    
    ChangeDir {
        path: String,
    },
    
    DeleteFiles {
        first: bool,
        where_clause: Vec<WhereClause>,
    },

    Exists {
        where_clause: Vec<WhereClause>,
    },

    Show,
}


fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn literal(input: &str) -> IResult<&str, &str> {
    delimited(char('\''), take_while1(|c| c != '\''), char('\''))(input)
}

fn asterisk(input: &str) -> IResult<&str, &str> {
    tag_no_case("*")(input)
}

fn column_identifier(input: &str) -> IResult<&str, &str> {
    alt((asterisk, identifier))(input)
}

fn column_list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(ws(char(',')), ws(column_identifier))(input)
}

fn where_clause(input: &str) -> IResult<&str, Vec<(&str, &str, &str)>> {
    separated_list0(ws(tag_no_case("AND")), ws(comparison))(input)
}

fn exists_statement(input: &str) -> IResult<&str, (&str, Vec<(&str, &str, &str)>)> {
    tuple((
        ws(tag_no_case("EXISTS")),
        where_clause,
    ))(input)
}

fn show_statement(input: &str) -> IResult<&str, &str> {
    ws(tag_no_case("SHOW"))(input)
}

fn operator(input: &str) -> IResult<&str, &str> {
    alt((
        tag("="),
        tag("<>"),
        tag("!="),
        tag("<"),
        tag("<="),
        tag(">"),
        tag(">="),
    ))(input)
}

fn comparison(input: &str) -> IResult<&str, (&str, &str, &str)> {
    tuple((ws(identifier), ws(operator), ws(literal)))(input)
}

fn select_statement(input: &str) -> IResult<&str, (&str, Vec<&str>, Option<Vec<(&str, &str, &str)>>, Option<Vec<&str>>)> {
    tuple((
        ws(tag_no_case("SELECT")),
        column_list,
        opt(preceded(ws(tag_no_case("WHERE")), where_clause)),
        opt(preceded(ws(tag_no_case("ORDER")), preceded(ws(tag_no_case("BY")), column_list))),
    ))(input)
}


fn directory_path(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanumeric() || c == '/' || c == '.' || c == '_')(input)
}

fn cd_statement(input: &str) -> IResult<&str, (&str, &str)> {
    tuple((
        ws(tag_no_case("CD")).or(ws(tag_no_case("CHANGEDIR"))),
        ws(directory_path),
    ))(input)
}

fn where_clause_to_enum(wh: Option<Vec<(&str, &str, &str)>>) -> Option<Vec<WhereClause>> {
    wh.map(|v| {
        v.into_iter().map(|(col, op, val)| {
            match op {
                "=" => WhereClause::Equal(col.to_string(), val.to_string()),
                "<>" | "!=" => WhereClause::NotEqual(col.to_string(), val.to_string()),
                "<" => WhereClause::LessThan(col.to_string(), val.to_string()),
                "<=" => WhereClause::LessThanOrEqual(col.to_string(), val.to_string()),
                ">" => WhereClause::GreaterThan(col.to_string(), val.to_string()),
                ">=" => WhereClause::GreaterThanOrEqual(col.to_string(), val.to_string()),
                _ => WhereClause::UnknownOperator(col.to_string(), val.to_string()),
            }
        }).collect()
    })
}

fn command(input: &str) -> IResult<&str, Command> {
    alt((
        map(select_statement, |(_select, columns, where_clause, order_by)| {
            Command::Select {
                columns: columns.iter().map(|&s| s.to_string()).collect(),
                order_by: order_by.map(|v| v.iter().map(|&s| s.to_string()).collect()),
                where_clause: where_clause_to_enum(where_clause)
            }
        }),
        map(cd_statement, |(_command, path)| {
            Command::ChangeDir {
                path: path.to_string(),
            }
        }),
        map(show_statement, |_command| {
            Command::Show
        }),
        map(exists_statement, |(_command, where_clause)|{
            Command::Exists { 
                where_clause: where_clause_to_enum(Some(where_clause)).unwrap_or_default(),
             }
        })
    ))(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<Command>> {
    separated_list0(ws(char(';')), ws(command))(input)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_statement() {
        let input = "SELECT * WHERE name = 'file_name.txt'";
        let expected = Command::Select {
            columns: vec!["*".to_string()],
            where_clause: Some(vec![WhereClause::Equal("name".to_string(), "file_name.txt".to_string())]),
            order_by: None,
        };

        let result = parse(input);
        assert_eq!(result, Ok(("", vec![expected])));
    }

    #[test]
    fn test_cd_statement() {
        let input = "CD /path/to/dir";
        let expected = Command::ChangeDir {
            path: "/path/to/dir".to_string(),
        };

        let result = parse(input);
        assert_eq!(result, Ok(("", vec![expected])));
    }

    #[test]
    fn test_show_statement() {
        let input = "SHOW";
        let expected = Command::Show;

        let result = parse(input);
        assert_eq!(result, Ok(("", vec![expected])));
    }
}
