use crate::StringId;

use std::collections::HashMap;

// Pre-loaded strings with known values
pub mod str {
    pub const EXCL: usize = 0;
    pub const EQ: usize = 1;
    pub const IS: usize = 2;
    pub const GT: usize = 3;
    pub const GE: usize = 4;
    pub const LT: usize = 5;
    pub const LE: usize = 6;
    pub const ANE: usize = 7;
    pub const AEQ: usize = 8;
    pub const ADD: usize = 9;
    pub const SUB: usize = 10;
    pub const MUL: usize = 11;
    pub const DIV: usize = 12;
    pub const DOT: usize = 13;
    pub const NIL: usize = 14;
    pub const STAT: usize = 15;
    pub const UNDERSCORE: usize = 16;
    pub const INTDIV: usize = 17;
    pub const MOD: usize = 18;
    pub const RSHIFT: usize = 19;
    pub const LSHIFT: usize = 20;
    pub const VAR: usize = 21;
    pub const INTEGER: usize = 22;
    pub const FLOAT: usize = 23;
    pub const ATOM: usize = 24;
    pub const COMPOUND: usize = 25;
    pub const NUMBER: usize = 26;
}

pub struct StringMap {
    map: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl Default for StringMap {
    fn default() -> Self {
        let reverse = vec![
            "!".to_string(),
            "=".to_string(),
            "is".to_string(),
            ">".to_string(),
            ">=".to_string(),
            "<".to_string(),
            "=<".to_string(),
            "=\\=".to_string(),
            "=:=".to_string(),
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            ".".to_string(),
            "[]".to_string(),
            "statistics".to_string(),
            "_".to_string(),
            "//".to_string(),
            "mod".to_string(),
            ">>".to_string(),
            "<<".to_string(),
            "var".to_string(),
            "integer".to_string(),
            "float".to_string(),
            "atom".to_string(),
            "compound".to_string(),
            "number".to_string(),
        ];

        let map = reverse
            .iter()
            .enumerate()
            .map(|(a, b)| (b.clone(), a))
            .collect();

        StringMap { map, reverse }
    }
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
