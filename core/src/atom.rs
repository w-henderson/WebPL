use crate::stringmap::StringMap;
use crate::{ast, StringId};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Atom {
    String(StringId),
    CharList(StringId, usize),
    Integer(i64),
    Float(f64),
}

impl Atom {
    pub fn new(string_map: &mut StringMap, atom: &ast::Atom) -> Self {
        match atom {
            ast::Atom::String(s) => Atom::String(string_map.alloc(s)),
            ast::Atom::CharList(s) => Atom::CharList(string_map.alloc(s), 0),
            ast::Atom::Integer(n) => Atom::Integer(*n),
            ast::Atom::Float(n) => Atom::Float(*n),
        }
    }

    pub fn to_string(self, string_map: &StringMap) -> String {
        match self {
            Atom::String(id) => string_map.get(id).unwrap().to_string(),
            Atom::CharList(id, index) => string_map.get(id).unwrap().chars().skip(index).collect(),
            Atom::Integer(integer) => integer.to_string(),
            Atom::Float(float) => float.to_string(),
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Atom::String(crate::stringmap::str::NIL))
    }
}
