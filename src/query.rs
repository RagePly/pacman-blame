use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    InvalidProperty(String),
    SyntaxError,
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use ParseError::*;
        match self {
            InvalidProperty(prop) => write!(f, "property not supported: {}", prop),
            SyntaxError => write!(f, "invalid syntax"),
        }
    }
}

#[derive(Debug)]
pub enum Query {
    PackageName(String),
}

impl Query {
    pub fn parse<S>(query: &S) -> Result<Query, ParseError>
    where
        S: AsRef<str>,
    {
        let Some((prop, value)) = query.as_ref().split_once(":") else {
            return Ok(Query::PackageName(query.as_ref().to_string()));
        };

        match prop {
            "package" => Ok(Query::PackageName(value.to_string())),
            _ if prop.trim() != prop => Err(ParseError::SyntaxError),
            _ => Err(ParseError::InvalidProperty(prop.to_string())),
        }
    }
}
