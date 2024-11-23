use crate::ast::StringMap;
use crate::atom::Atom;
use crate::{CodeTerm, HeapTerm, HeapTermPtr, Query, StringId};

#[derive(Default)]
pub struct VarArena {
    arena: Vec<HeapTerm>,
    string_map: StringMap,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(usize);

impl VarArena {
    pub fn new(
        string_map: StringMap,
        query: &Query,
    ) -> (Self, Vec<HeapTermPtr>, Vec<(String, HeapTermPtr)>) {
        let mut arena = Self {
            arena: Vec::new(),
            string_map,
        };

        let mut var_map = Vec::new();
        let mut heap_query = Vec::new();

        for term in query {
            heap_query.push(arena.alloc(term, &mut var_map));
        }

        let var_map = var_map
            .into_iter()
            .map(|(id, ptr)| (arena.string_map.get(id).unwrap().to_string(), ptr))
            .collect();

        (arena, heap_query, var_map)
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.arena.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.arena.truncate(checkpoint.0);
    }

    pub fn alloc(
        &mut self,
        term: &CodeTerm,
        var_map: &mut Vec<(StringId, HeapTermPtr)>,
    ) -> HeapTermPtr {
        let result = self.arena.len();

        match term {
            CodeTerm::Atom(atom) => self.arena.push(HeapTerm::Atom(*atom)),
            CodeTerm::Var(id) => {
                if let Some((_, unified)) = var_map.iter().find(|(x, _)| x == id) {
                    self.arena.push(HeapTerm::Var(*unified));
                } else {
                    self.arena.push(HeapTerm::Var(result));
                    var_map.push((*id, result));
                }
            }
            CodeTerm::Compound(functor, args) => {
                let arity = args.len();

                self.arena.push(HeapTerm::Compound(*functor, arity, None));

                let mut next = None;
                for arg in args.iter().rev() {
                    let head = self.alloc(arg, var_map);
                    let tail = next.replace(self.arena.len());
                    self.arena.push(HeapTerm::CompoundCons(head, tail));
                }

                if let HeapTerm::Compound(_, _, arg) = &mut self.arena[result] {
                    *arg = next;
                }
            }
        }

        result
    }

    pub fn alloc_atom(&mut self, atom: Atom) -> HeapTermPtr {
        let result = self.arena.len();
        self.arena.push(HeapTerm::Atom(atom));
        result
    }

    pub fn get(&self, mut var: HeapTermPtr) -> &HeapTerm {
        debug_assert!(var < self.arena.len());

        // Follow the chain of variable bindings until we reach the root.
        loop {
            match &self.arena[var] {
                HeapTerm::Var(x) if *x != var => var = *x,
                _ => return &self.arena[var],
            }
        }
    }

    pub fn unify(&mut self, a: usize, b: usize) {
        match &mut self.arena[a] {
            HeapTerm::Var(x) => *x = b,
            _ => unreachable!(),
        }
    }

    pub fn unbind(&mut self, term: HeapTermPtr) {
        match &mut self.arena[term] {
            HeapTerm::Var(x) => *x = term,
            _ => unreachable!(),
        }
    }

    pub fn get_atom(&self, atom: StringId) -> &str {
        self.string_map.get(atom).unwrap()
    }

    pub fn serialize(&self, term: HeapTermPtr, name: &str) -> String {
        self.serialize_inner(term, name, &mut Vec::new(), None)
    }

    fn serialize_inner(
        &self,
        term: HeapTermPtr,
        name: &str,
        stack: &mut Vec<HeapTermPtr>,
        equal_limit: Option<usize>,
    ) -> String {
        match &self.arena[term] {
            HeapTerm::Atom(atom) => atom.to_string(&self.string_map),
            HeapTerm::Var(x) => {
                let position = stack.iter().position(|y| y == x);

                if *x == term
                    || (equal_limit.is_some()
                        && position.map(|p| p >= equal_limit.unwrap()).unwrap_or(false))
                {
                    return format!("_{}", x);
                }

                if position.is_some() {
                    return name.to_string();
                }

                stack.push(term);
                self.serialize_inner(*x, name, stack, equal_limit)
            }
            HeapTerm::Compound(functor, _, next) => {
                stack.push(term);

                let len = stack.len();

                let mut result = format!("{}(", self.get_atom(*functor));
                let mut next = *next;

                while let Some(arg) = next {
                    if let HeapTerm::CompoundCons(head, tail) = &self.arena[arg] {
                        result.push_str(&self.serialize_inner(*head, name, stack, equal_limit));
                        stack.truncate(len);
                        next = *tail;
                    }

                    if next.is_some() {
                        result.push_str(", ");
                    }
                }

                result.push(')');

                result
            }
            HeapTerm::CompoundCons(_, _) => unreachable!(),
        }
    }
}
