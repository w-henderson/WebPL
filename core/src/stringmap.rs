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
            "<=".to_string(),
            "=\\=".to_string(),
            "=:=".to_string(),
            "+".to_string(),
            "-".to_string(),
            "*".to_string(),
            "/".to_string(),
            ".".to_string(),
            "[]".to_string(),
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
