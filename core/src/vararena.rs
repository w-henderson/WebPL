use crate::{Clause, CodeTerm, HeapTerm, HeapTermPtr, Query, VarName};

#[derive(Default)]
pub struct VarArena(Vec<HeapTerm>);

pub struct Checkpoint(usize);

impl VarArena {
    pub fn new(query: &Query) -> (Self, Vec<HeapTermPtr>, Vec<(VarName, HeapTermPtr)>) {
        let mut arena = Self::default();
        let mut var_map = Vec::new();
        let mut heap_query = Vec::new();

        for term in query {
            heap_query.push(arena.alloc(term, &mut var_map));
        }

        (arena, heap_query, var_map)
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.0.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.0.truncate(checkpoint.0);
    }

    pub fn alloc(&mut self, term: &CodeTerm, var_map: &mut Vec<(VarName, usize)>) -> HeapTermPtr {
        let result = self.0.len();

        match term {
            CodeTerm::Atom(atom) => self.0.push(HeapTerm::Atom(atom.clone())),
            CodeTerm::Var(id) => {
                if let Some((_, unified)) = var_map.iter().find(|(x, _)| x == id) {
                    self.0.push(HeapTerm::Var(*unified));
                } else {
                    self.0.push(HeapTerm::Var(result));
                    var_map.push((id.clone(), result));
                }
            }
            CodeTerm::Compound(functor, args) => {
                let arity = args.len();
                self.0.push(HeapTerm::Compound(functor.clone(), arity));

                for arg in args {
                    self.alloc(arg, var_map);
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

    pub fn get(&self, mut var: HeapTermPtr) -> &HeapTerm {
        debug_assert!(var < self.0.len());

        // Follow the chain of variable bindings until we reach the root.
        loop {
            match &self.0[var] {
                HeapTerm::Var(x) if *x != var => var = *x,
                _ => return &self.0[var],
            }
        }
    }

    pub fn unify(&mut self, a: usize, b: usize) {
        match &mut self.0[a] {
            HeapTerm::Var(x) => *x = b,
            _ => unreachable!(),
        }
    }

    pub fn unbind(&mut self, term: HeapTermPtr) {
        match &mut self.0[term] {
            HeapTerm::Var(x) => *x = term,
            _ => unreachable!(),
        }
    }

    pub fn serialize(&self, term: HeapTermPtr) -> String {
        match &self.0[term] {
            HeapTerm::Atom(atom) => atom.clone(),
            HeapTerm::Var(x) if *x == term => format!("_{}", x),
            HeapTerm::Var(x) => self.serialize(*x),
            HeapTerm::Compound(functor, arity) => {
                let args = (0..*arity)
                    .map(|i| self.serialize(term + 1 + i))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", functor, args)
            }
        }
    }
}
