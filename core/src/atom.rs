use crate::stringmap::StringMap;
use crate::StringId;

#[derive(PartialEq, Copy, Clone)]
pub enum Atom {
    String(StringId),
    Integer(i64),
    Float(f64),
}

impl Atom {
    pub fn new(string_map: &mut StringMap, atom: &str) -> Self {
        if let Ok(integer) = atom.parse::<i64>() {
            Atom::Integer(integer)
        } else if let Ok(float) = atom.parse::<f64>() {
            Atom::Float(float)
        } else {
            Atom::String(string_map.alloc(atom))
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
