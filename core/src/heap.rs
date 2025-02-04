use crate::atom::Atom;
use crate::stringmap::StringMap;
use crate::{ChoicePointIdx, ClauseName, CodeTerm, HeapTerm, HeapTermPtr, Query, StringId};

pub struct Heap {
    pub(crate) data: Vec<HeapTerm>,
    pub(crate) string_map: StringMap,
    pub(crate) initialised: bool,
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
            initialised: false,
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

        heap.initialised = true;

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
            CodeTerm::Var(crate::stringmap::str::UNDERSCORE) => {
                self.data.push(HeapTerm::Var(result, false));
            }
            CodeTerm::Var(id) => {
                if let Some((_, unified)) = var_map.iter().find(|(x, _)| x == id) {
                    self.data.push(HeapTerm::Var(*unified, true)); // shunted
                } else {
                    self.data.push(HeapTerm::Var(result, false));
                    var_map.push((*id, result));
                }
            }
            CodeTerm::Compound(functor, args) => {
                let arity = args.len();

                self.data.push(HeapTerm::Compound(*functor, arity));

                let args_heap = self.data.len();
                for _ in args {
                    self.data.push(HeapTerm::Var(0, false));
                }

                for (i, arg) in args.iter().enumerate() {
                    let arg = self.alloc(arg, var_map, choice_point_idx);
                    if let HeapTerm::Var(x, _) = &mut self.data[args_heap + i] {
                        *x = arg;
                    }
                }
            }
            CodeTerm::Cut => self.data.push(HeapTerm::Cut(choice_point_idx)),
            CodeTerm::Lambda(js, args) => {
                let arity = args.len();

                self.data.push(HeapTerm::Lambda(*js, arity));

                let args_heap = self.data.len();
                for _ in args {
                    self.data.push(HeapTerm::Var(0, false));
                }

                for (i, arg) in args.iter().enumerate() {
                    let arg = self.alloc(arg, var_map, choice_point_idx);
                    if let HeapTerm::Var(x, _) = &mut self.data[args_heap + i] {
                        *x = arg;
                    }
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

    pub fn get(&self, var: HeapTermPtr) -> &HeapTerm {
        &self.data[self.get_ptr(var)]
    }

    pub fn get_ptr(&self, mut var: HeapTermPtr) -> HeapTermPtr {
        debug_assert!(var < self.data.len());

        // Follow the chain of variable bindings until we reach the root.
        loop {
            match &self.data[var] {
                HeapTerm::Var(x, _) if *x != var => var = *x,
                _ => return var,
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
            HeapTerm::Compound(functor, arity) => ClauseName(*functor, *arity),
            HeapTerm::Cut(_) => ClauseName(crate::stringmap::str::EXCL, 0),
            HeapTerm::Lambda(code, arity) => ClauseName(*code, *arity),
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
