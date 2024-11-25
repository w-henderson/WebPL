use crate::stringmap::StringMap;
use crate::{atom, CodeTerm, ToCodeTerm};

pub struct Program(pub Vec<Clause>);

pub struct Query(pub Vec<Predicate>);

pub struct Clause(pub Predicate, pub Vec<Predicate>);

pub enum Predicate {
    Atom(Atom),
    Compound(String, Vec<Term>),
}

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

impl ToCodeTerm for Predicate {
    fn to_code_term(&self, string_map: &mut StringMap) -> CodeTerm {
        match self {
            Predicate::Atom(atom) => CodeTerm::Atom(atom::Atom::new(string_map, atom)),
            Predicate::Compound(functor, args) => CodeTerm::Compound(
                string_map.alloc(functor),
                args.iter()
                    .map(|arg| arg.to_code_term(string_map))
                    .collect(),
            ),
        }
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
