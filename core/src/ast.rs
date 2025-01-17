use crate::stringmap::StringMap;
use crate::{atom, ClauseName, CodeTerm, Error, ErrorLocation};

use lalrpop_util::lexer::Token;

pub struct Program(pub Vec<Clause>);

pub struct Query(pub Vec<Term>);

pub struct Clause(pub Term, pub Vec<Term>);

pub enum Term {
    Atom(Atom),
    Variable(String),
    Compound(String, Vec<Term>),
    Cut,
}

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

    pub fn to_code_term(&self, string_map: &mut StringMap) -> (CodeTerm, Option<ClauseName>) {
        match self {
            Term::Atom(atom) => {
                let atom = atom::Atom::new(string_map, atom);
                if let atom::Atom::String(string_id) = &atom {
                    (CodeTerm::Atom(atom), Some(ClauseName(*string_id, 0)))
                } else {
                    (CodeTerm::Atom(atom), None)
                }
            }
            Term::Variable(var) => (CodeTerm::Var(string_map.alloc(var)), None),
            Term::Compound(functor, args) => {
                let functor = string_map.alloc(functor);
                (
                    CodeTerm::Compound(
                        functor,
                        args.iter()
                            .map(|arg| arg.to_code_term(string_map).0)
                            .collect(),
                    ),
                    Some(ClauseName(functor, args.len())),
                )
            }
            Term::Cut => (CodeTerm::Cut, None),
        }
    }
}

pub fn parse_error(input: &str, error: lalrpop_util::ParseError<usize, Token<'_>, &str>) -> Error {
    match error {
        lalrpop_util::ParseError::InvalidToken { location } => {
            with_location("Invalid token".into(), input, location)
        }
        lalrpop_util::ParseError::UnrecognizedEof {
            location,
            expected: _,
        } => with_location(
            "Unexpected end of file, did you forget a '.'?".into(),
            input,
            location,
        ),
        lalrpop_util::ParseError::UnrecognizedToken { token, expected: _ } => with_location(
            format!("Unexpected token `{}`", &input[token.0..token.2]),
            input,
            token.0,
        ),
        lalrpop_util::ParseError::ExtraToken { token } => with_location(
            format!("Extra token `{}`", &input[token.0..token.2]),
            input,
            token.0,
        ),
        lalrpop_util::ParseError::User { error } => Error {
            location: None,
            error: error.to_string(),
        },
    }
}

fn with_location(error: String, input: &str, offset: usize) -> Error {
    Error {
        location: Some(get_location(input, offset)),
        error,
    }
}

fn get_location(input: &str, offset: usize) -> ErrorLocation {
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
        offset,
        line,
        column,
    }
}
