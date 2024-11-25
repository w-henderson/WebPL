use crate::stringmap::StringMap;
use crate::{atom, CodeTerm, ToCodeTerm};

pub struct Program(pub Vec<Clause>);

pub struct Query(pub Vec<Term>);

pub struct Clause(pub Term, pub Vec<Term>);

pub enum Term {
    Atom(Atom),
    Variable(String),
    Compound(String, Vec<Term>),
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
}

impl ToCodeTerm for Term {
    fn to_code_term(&self, string_map: &mut StringMap) -> CodeTerm {
        match self {
            Term::Atom(atom) => CodeTerm::Atom(atom::Atom::new(string_map, atom)),
            Term::Variable(var) => CodeTerm::Var(string_map.alloc(var)),
            Term::Compound(functor, args) => CodeTerm::Compound(
                string_map.alloc(functor),
                args.iter()
                    .map(|arg| arg.to_code_term(string_map))
                    .collect(),
            ),
        }
    }
}
