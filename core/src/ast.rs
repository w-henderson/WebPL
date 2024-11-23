use crate::{AtomId, CodeTerm};

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
    pub atom_map: BiMap,
    pub variable_map: BiMap,
}

#[derive(Default)]
pub struct BiMap {
    map: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl BiMap {
    pub fn alloc(&mut self, atom: &str) -> AtomId {
        if let Some(ptr) = self.map.get(atom) {
            *ptr
        } else {
            let ptr = self.reverse.len();
            self.map.insert(atom.to_string(), ptr);
            self.reverse.push(atom.to_string());
            ptr
        }
    }

    pub fn get(&self, ptr: AtomId) -> Option<&str> {
        self.reverse.get(ptr).map(|s| s.as_str())
    }
}

impl ASTTerm {
    pub fn to_code_term(&self, string_map: &mut StringMap) -> CodeTerm {
        match self {
            ASTTerm::Atom(atom) => CodeTerm::Atom(string_map.atom_map.alloc(atom)),
            ASTTerm::Var(var) => CodeTerm::Var(string_map.variable_map.alloc(var)),
            ASTTerm::Compound(functor, args) => CodeTerm::Compound(
                string_map.atom_map.alloc(functor),
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
