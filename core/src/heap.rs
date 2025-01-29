use crate::atom::Atom;
use crate::stringmap::StringMap;
use crate::{ChoicePointIdx, ClauseName, CodeTerm, HeapTerm, HeapTermPtr, Query, StringId};

pub struct Heap {
    pub(crate) data: Vec<HeapTerm>,
    pub(crate) string_map: StringMap,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(pub usize);

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
                    self.data.push(HeapTerm::Var(*unified, false));
                } else {
                    self.data.push(HeapTerm::Var(result, false));
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
            CodeTerm::Lambda(js, args) => {
                let arity = args.len();

                self.data.push(HeapTerm::Lambda(*js, arity, None));

                let mut next = None;
                for arg in args.iter().rev() {
                    let head = self.alloc(arg, var_map, choice_point_idx);
                    let tail = next.replace(self.data.len());
                    self.data.push(HeapTerm::CompoundCons(head, tail));
                }

                if let HeapTerm::Lambda(_, _, arg) = &mut self.data[result] {
                    *arg = next;
                }
            }
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
                HeapTerm::Var(x, _) if *x != var => var = *x,
                _ => return &self.data[var],
            }
        }
    }

    pub fn unify(&mut self, a: HeapTermPtr, b: HeapTermPtr) {
        match &mut self.data[a] {
            HeapTerm::Var(x, _) => *x = b,
            _ => unreachable!(),
        }
    }

    pub fn unbind(&mut self, term: HeapTermPtr) {
        match &mut self.data[term] {
            HeapTerm::Var(x, _) => *x = term,
            _ => unreachable!(),
        }
    }

    pub fn mark_shunted(&mut self, term: HeapTermPtr) {
        match &mut self.data[term] {
            HeapTerm::Var(_, shunted) => *shunted = true,
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
            HeapTerm::Lambda(code, arity, _) => ClauseName(*code, *arity),
            _ => unreachable!(),
        }
    }

    pub fn size(&self) -> usize {
        self.data.len() * std::mem::size_of::<HeapTerm>()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<HeapTerm>()
    }
}
