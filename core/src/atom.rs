use crate::stringmap::StringMap;
use crate::{ast, StringId};

#[derive(PartialEq, Copy, Clone)]
pub enum Atom {
    String(StringId),
    Integer(i64),
    Float(f64),
}

impl Atom {
    pub fn new(string_map: &mut StringMap, atom: &ast::Atom) -> Self {
        match atom {
            ast::Atom::String(s) => Atom::String(string_map.alloc(s)),
            ast::Atom::Integer(n) => Atom::Integer(*n),
            ast::Atom::Float(n) => Atom::Float(*n),
        }
    }

    pub fn to_string(self, string_map: &StringMap) -> String {
        match self {
            Atom::String(id) => string_map.get(id).unwrap().to_string(),
            Atom::Integer(integer) => integer.to_string(),
            Atom::Float(float) => float.to_string(),
        }
    }
}
