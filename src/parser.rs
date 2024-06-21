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
pub enum Ordering {
    Ascending,
    Descending,
}

#[derive(Debug, PartialEq)]
pub enum WhereType<'a> {
    Conditions(Vec<(&'a str, &'a str, &'a str)>),
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Select {
        props: Vec<String>,
        where_clause: Option<Vec<WhereClause>>,
        order_by: Option<Vec<String>>,
        limit: Option<usize>,
        from_path: Option<String>,
        ordering: Option<Ordering>,
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
    // example => "name" or "file_name"
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn limit_statement(input: &str) -> IResult<&str, usize> {
    preceded(ws(tag_no_case("LIMIT")), ws(take_while1(|c: char| c.is_numeric())))(input).map(|(remaining, limit)| {
        (remaining, limit.parse().unwrap())
    })
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn literal(input: &str) -> IResult<&str, &str> {
    // literals like -> 'file_name.txt'
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


fn from_path_clause(input: &str) -> IResult<&str, &str> {
    preceded(ws(tag_no_case("FROM")), ws(directory_path))(input)
}


fn ordering_clause(input: &str) -> IResult<&str, Ordering> {
    alt((
        map(ws(tag_no_case("ASC")), |_| Ordering::Ascending),
        map(ws(tag_no_case("DESC")), |_| Ordering::Descending),
    ))(input)
}


fn select_statement(input: &str) -> IResult<&str, (&str, Vec<&str>, Option<Vec<(&str, &str, &str)>>, Option<Vec<&str>>, Option<usize>, Option<&str>, Option<Ordering>)> {
    tuple((
        ws(tag_no_case("SELECT")),
        column_list,
        opt(preceded(ws(tag_no_case("WHERE")), where_clause)),
        opt(preceded(ws(tag_no_case("ORDER")), preceded(ws(tag_no_case("BY")), column_list))),
        opt(limit_statement),
        opt(from_path_clause),
        opt(ordering_clause)
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
        map(select_statement, |(select)| {
            let (_command, columns, where_clause, order_by, _limit, _from_path, _ordering) = select;
            Command::Select {
                props: columns.iter().map(|&s| s.to_string()).collect(),
                order_by: order_by.map(|v| v.iter().map(|&s| s.to_string()).collect()),
                where_clause: where_clause_to_enum(where_clause),
                limit: _limit,
                from_path: _from_path.map(|s| s.to_string()),
                ordering: _ordering,
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
            props: vec!["*".to_string()],
            where_clause: Some(vec![WhereClause::Equal("name".to_string(), "file_name.txt".to_string())]),
            order_by: None,
            limit: None,
            from_path: None,
            ordering: None,
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
