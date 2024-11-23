use crate::atom::Atom;
use crate::{CodeTerm, StringId};

use std::collections::HashMap;

pub enum ASTTerm {
    Atom(String),
    Var(String),
    Compound(String, Vec<ASTTerm>),
}

pub type Clause = (ASTTerm, Vec<ASTTerm>);
pub type Query = Vec<ASTTerm>;
pub type Program = Vec<Clause>;

#[derive(Default)]
pub struct StringMap {
    map: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl StringMap {
    pub fn alloc(&mut self, atom: &str) -> StringId {
        if let Some(ptr) = self.map.get(atom) {
            *ptr
        } else {
            let ptr = self.reverse.len();
            self.map.insert(atom.to_string(), ptr);
            self.reverse.push(atom.to_string());
            ptr
        }
    }

    pub fn get(&self, ptr: StringId) -> Option<&str> {
        self.reverse.get(ptr).map(|s| s.as_str())
    }
}

impl ASTTerm {
    pub fn to_code_term(&self, string_map: &mut StringMap) -> CodeTerm {
        match self {
            ASTTerm::Atom(atom) => CodeTerm::Atom(Atom::new(string_map, atom)),
            ASTTerm::Var(var) => CodeTerm::Var(string_map.alloc(var)),
            ASTTerm::Compound(functor, args) => CodeTerm::Compound(
                string_map.alloc(functor),
                args.iter()
                    .map(|arg| arg.to_code_term(string_map))
                    .collect(),
            ),
        }
    }
}

pub fn to_code_term(program: Program, query: Query) -> (crate::Program, crate::Query, StringMap) {
    let mut string_map = StringMap::default();

    let program = program
        .into_iter()
        .map(|(head, body)| {
            (
                head.to_code_term(&mut string_map),
                body.into_iter()
                    .map(|term| term.to_code_term(&mut string_map))
                    .collect(),
            )
        })
        .collect();

    let query = query
        .into_iter()
        .map(|term| term.to_code_term(&mut string_map))
        .collect();

    (program, query, string_map)
}
