use crate::stringmap::StringMap;
use crate::{atom, ClauseName, CodeTerm};

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
