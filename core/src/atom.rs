use std::collections::HashMap;

use crate::HeapAtomPtr;

#[derive(Default)]
pub struct AtomMap {
    map: HashMap<String, HeapAtomPtr>,
    reverse: Vec<String>,
}

impl AtomMap {
    pub fn alloc(&mut self, atom: &str) -> HeapAtomPtr {
        if let Some(ptr) = self.map.get(atom) {
            *ptr
        } else {
            let ptr = self.reverse.len();
            self.map.insert(atom.to_string(), ptr);
            self.reverse.push(atom.to_string());
            ptr
        }
    }

    pub fn get(&self, ptr: HeapAtomPtr) -> Option<&str> {
        self.reverse.get(ptr).map(|s| s.as_str())
    }
}
