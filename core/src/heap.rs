use crate::atom::Atom;
use crate::stringmap::StringMap;
use crate::{ChoicePointIdx, ClauseName, CodeTerm, HeapTerm, HeapTermPtr, Query, StringId};

pub struct Heap {
    data: Vec<HeapTerm>,
    string_map: StringMap,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(usize);

impl Heap {
    pub fn new(
        string_map: StringMap,
        query: &Query,
    ) -> (Self, Vec<HeapTermPtr>, Vec<(String, HeapTermPtr)>) {
        let mut heap = Self {
            data: Vec::new(),
            string_map,
        };

        let mut var_map = Vec::new();
        let mut heap_query = Vec::new();

        for term in query {
            heap_query.push(heap.alloc(term, &mut var_map, 0));
        }

        let var_map = var_map
            .into_iter()
            .map(|(id, ptr)| (heap.string_map.get(id).unwrap().to_string(), ptr))
            .collect();

        (heap, heap_query, var_map)
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.data.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.data.truncate(checkpoint.0);
    }

    pub fn alloc(
        &mut self,
        term: &CodeTerm,
        var_map: &mut Vec<(StringId, HeapTermPtr)>,
        choice_point_idx: ChoicePointIdx,
    ) -> HeapTermPtr {
        let result = self.data.len();

        match term {
            CodeTerm::Atom(atom) => self.data.push(HeapTerm::Atom(*atom)),
            CodeTerm::Var(id) => {
                if let Some((_, unified)) = var_map.iter().find(|(x, _)| x == id) {
                    self.data.push(HeapTerm::Var(*unified));
                } else {
                    self.data.push(HeapTerm::Var(result));
                    var_map.push((*id, result));
                }
            }
            CodeTerm::Compound(functor, args) => {
                let arity = args.len();

                self.data.push(HeapTerm::Compound(*functor, arity, None));

                let mut next = None;
                for arg in args.iter().rev() {
                    let head = self.alloc(arg, var_map, choice_point_idx);
                    let tail = next.replace(self.data.len());
                    self.data.push(HeapTerm::CompoundCons(head, tail));
                }

                if let HeapTerm::Compound(_, _, arg) = &mut self.data[result] {
                    *arg = next;
                }
            }
            CodeTerm::Cut => self.data.push(HeapTerm::Cut(choice_point_idx)),
        }

        result
    }

    pub fn alloc_atom(&mut self, atom: Atom) -> HeapTermPtr {
        let result = self.data.len();
        self.data.push(HeapTerm::Atom(atom));
        result
    }

    pub fn get(&self, mut var: HeapTermPtr) -> &HeapTerm {
        debug_assert!(var < self.data.len());

        // Follow the chain of variable bindings until we reach the root.
        loop {
            match &self.data[var] {
                HeapTerm::Var(x) if *x != var => var = *x,
                _ => return &self.data[var],
            }
        }
    }

    pub fn unify(&mut self, a: usize, b: usize) {
        match &mut self.data[a] {
            HeapTerm::Var(x) => *x = b,
            _ => unreachable!(),
        }
    }

    pub fn unbind(&mut self, term: HeapTermPtr) {
        match &mut self.data[term] {
            HeapTerm::Var(x) => *x = term,
            _ => unreachable!(),
        }
    }

    pub fn get_atom(&self, atom: StringId) -> &str {
        self.string_map.get(atom).unwrap()
    }

    pub fn get_name(&self, term: HeapTermPtr) -> ClauseName {
        match self.get(term) {
            HeapTerm::Atom(Atom::String(name)) => ClauseName(*name, 0),
            HeapTerm::Compound(functor, arity, _) => ClauseName(*functor, *arity),
            HeapTerm::Cut(_) => ClauseName(crate::stringmap::str::EXCL, 0),
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
        match &self.data[term] {
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
                    if let HeapTerm::CompoundCons(head, tail) = &self.data[arg] {
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
            HeapTerm::Cut(_) => "!".to_string(),
        }
    }
}
