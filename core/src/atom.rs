use crate::stringmap::{self, StringMap};
use crate::{ast, StringId};

#[derive(Copy, Clone, Debug)]
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

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(stringmap::str::NIL), Self::CharList(stringmap::str::EMPTY, _))
            | (Self::CharList(stringmap::str::EMPTY, _), Self::String(stringmap::str::NIL)) => true,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::CharList(l0, l1), Self::CharList(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            _ => false,
        }
    }
}
