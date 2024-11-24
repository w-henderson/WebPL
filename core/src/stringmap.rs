use crate::StringId;

use std::collections::HashMap;

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
