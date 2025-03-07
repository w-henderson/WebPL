use crate::{Error, ErrorLocation};

use lalrpop_util::lexer::Token;

#[derive(Debug)]
pub struct Program(pub Vec<Clause>);

pub struct Query(pub Vec<Term>);

#[derive(Debug)]
pub struct Clause(pub Term, pub Vec<Term>);

#[derive(Debug)]
pub enum Term {
    Atom(Atom),
    Variable(String),
    Compound(String, Vec<Term>),
    Cut,
    Lambda(String, Vec<String>),
}

#[derive(Debug)]
pub enum Atom {
    String(String),
    Integer(i64),
    Float(f64),
}

impl Term {
    // Support syntactic sugar for lists.
    pub fn list(terms: Vec<Term>, tail: Option<Term>) -> Term {
        let mut term = tail.unwrap_or(Term::Atom(Atom::String("[]".to_string())));

        for t in terms.into_iter().rev() {
            term = Term::Compound(".".to_string(), vec![t, term]);
        }

        term
    }

    pub fn parse_lambda(js_str: &str) -> Result<Term, &'static str> {
        let js = js_str.as_bytes();

        let mut vars = Vec::new();

        let consume_whitespace = |mut start: usize| -> usize {
            while let Some(c) = js.get(start) {
                if !c.is_ascii_whitespace() {
                    break;
                }
                start += 1;
            }

            start
        };

        let read_var = |start: usize| -> Option<(String, usize)> {
            if !js.get(start)?.is_ascii_uppercase() {
                return None;
            }

            let mut end = start + 1;

            while js
                .get(end)
                .map(|c| c.is_ascii_alphanumeric() || *c == b'_')
                .unwrap_or(false)
            {
                end += 1;
            }

            Some((js_str[start..end].to_string(), end))
        };

        let mut i = consume_whitespace(2);

        if js.get(i) == Some(&b'(') {
            i = consume_whitespace(i + 1);
            while let Some((var, new_i)) = read_var(i) {
                vars.push(var);
                i = consume_whitespace(new_i);
                if *js.get(i).ok_or("Unexpected EOF")? == b',' {
                    i = consume_whitespace(i + 1);
                } else {
                    i = consume_whitespace(i);
                    break;
                }
            }

            if *js.get(i).ok_or("Unexpected EOF")? != b')' {
                return Err("Expected )");
            }

            i = consume_whitespace(i + 1);
        } else if let Some((var, new_i)) = read_var(i) {
            vars.push(var);
            i = consume_whitespace(new_i);
        } else {
            return Err("Expected ( or variable name");
        }

        if *js.get(i).ok_or("Unexpected EOF")? != b'='
            || *js.get(i + 1).ok_or("Unexpected EOF")? != b'>'
        {
            return Err("Expected =>");
        }

        i = consume_whitespace(i + 2);

        let js = if *js.get(i).ok_or("Unexpected EOF")? == b'{' {
            js_str[i..js_str.len() - 2].trim().to_string()
        } else {
            "return ".to_string() + js_str[i..js_str.len() - 2].trim()
        };

        Ok(Term::Lambda(js, vars))
    }
}

pub fn parse_error(
    input: &str,
    query: bool,
    error: lalrpop_util::ParseError<usize, Token<'_>, &str>,
) -> Error {
    match error {
        lalrpop_util::ParseError::InvalidToken { location } => {
            with_location("Invalid token".into(), input, query, location)
        }
        lalrpop_util::ParseError::UnrecognizedEof {
            location,
            expected: _,
        } => with_location(
            "Unexpected end of file, did you forget a '.'?".into(),
            input,
            query,
            location,
        ),
        lalrpop_util::ParseError::UnrecognizedToken { token, expected: _ } => with_location(
            format!("Unexpected token `{}`", &input[token.0..token.2]),
            input,
            query,
            token.0,
        ),
        lalrpop_util::ParseError::ExtraToken { token } => with_location(
            format!("Extra token `{}`", &input[token.0..token.2]),
            input,
            query,
            token.0,
        ),
        lalrpop_util::ParseError::User { error } => Error {
            location: None,
            error: error.to_string(),
        },
    }
}

fn with_location(error: String, input: &str, query: bool, offset: usize) -> Error {
    Error {
        location: Some(get_location(input, query, offset)),
        error,
    }
}

fn get_location(input: &str, query: bool, offset: usize) -> ErrorLocation {
    let mut line = 1;
    let mut column = 1;

    for (i, c) in input.chars().enumerate() {
        if i == offset {
            break;
        }

        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    ErrorLocation {
        query,
        offset,
        line,
        column,
    }
}
