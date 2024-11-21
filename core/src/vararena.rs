use crate::{Atom, Clause, CodeTerm, HeapTerm, HeapTermPtr, Query, VarName};

#[derive(Default)]
pub struct VarArena(Vec<HeapTerm>);

#[derive(Copy, Clone, Debug)]
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
                self.0.push(HeapTerm::Compound(functor.clone(), Vec::new()));

                let mut heap_args = Vec::with_capacity(arity);
                for arg in args {
                    heap_args.push(self.alloc(arg, var_map));
                }

                if let HeapTerm::Compound(_, ref mut args) = self.0[result] {
                    *args = heap_args;
                }

                return result;
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

    pub fn alloc_atom(&mut self, atom: Atom) -> HeapTermPtr {
        let result = self.0.len();
        self.0.push(HeapTerm::Atom(atom));
        result
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
        match &self.0[term] {
            HeapTerm::Atom(atom) => atom.clone(),
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
            HeapTerm::Compound(functor, args) => {
                stack.push(term);

                let len = stack.len();

                format!(
                    "{}({})",
                    functor,
                    args.iter()
                        .map(|arg| {
                            let result =
                                self.serialize_inner(*arg, name, stack, equal_limit.or(Some(len)));
                            stack.truncate(len);
                            result
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}
