use crate::atom::AtomMap;
use crate::{Clause, CodeTerm, HeapAtomPtr, HeapTerm, HeapTermPtr, Query};

#[derive(Default)]
pub struct VarArena {
    arena: Vec<HeapTerm>,
    atom_map: AtomMap,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(usize);

impl VarArena {
    pub fn new(query: &Query) -> (Self, Vec<HeapTermPtr>, Vec<(String, HeapTermPtr)>) {
        let mut arena = Self::default();
        let mut var_map = Vec::new();
        let mut heap_query = Vec::new();

        for term in query {
            heap_query.push(arena.alloc(term, &mut var_map));
        }

        (arena, heap_query, var_map)
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.arena.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.arena.truncate(checkpoint.0);
    }

    pub fn alloc(&mut self, term: &CodeTerm, var_map: &mut Vec<(String, usize)>) -> HeapTermPtr {
        let result = self.arena.len();

        match term {
            CodeTerm::Atom(atom) => self.arena.push(HeapTerm::Atom(self.atom_map.alloc(atom))),
            CodeTerm::Var(id) => {
                if let Some((_, unified)) = var_map.iter().find(|(x, _)| x == id) {
                    self.arena.push(HeapTerm::Var(*unified));
                } else {
                    self.arena.push(HeapTerm::Var(result));
                    var_map.push((id.clone(), result));
                }
            }
            CodeTerm::Compound(functor, args) => {
                let arity = args.len();

                self.arena.push(HeapTerm::Compound(
                    self.atom_map.alloc(functor),
                    arity,
                    None,
                ));

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

    pub fn alloc_clause(&mut self, clause: &Clause) -> (HeapTermPtr, Vec<HeapTermPtr>) {
        let (head, body) = clause;
        let mut var_map = Vec::new();

        (
            self.alloc(head, &mut var_map),
            body.iter()
                .map(|term| self.alloc(term, &mut var_map))
                .collect(),
        )
    }

    pub fn alloc_atom(&mut self, atom: String) -> HeapTermPtr {
        let result = self.arena.len();
        self.arena.push(HeapTerm::Atom(self.atom_map.alloc(&atom)));
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

    pub fn get_atom(&self, atom: HeapAtomPtr) -> &str {
        self.atom_map.get(atom).unwrap()
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
            HeapTerm::Atom(atom) => self.atom_map.get(*atom).unwrap().to_string(),
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
