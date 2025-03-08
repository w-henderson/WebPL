use crate::atom::Atom;
use crate::stringmap::StringMap;
use crate::{ChoicePointIdx, ClauseName, HeapClausePtr, HeapTerm, HeapTermPtr, StringId};

#[derive(Default)]
pub struct Heap {
    pub(crate) data: Vec<HeapTerm>,
    pub(crate) string_map: StringMap,
    pub(crate) code_end: HeapTermPtr,
}

#[derive(Copy, Clone, Debug)]
pub struct Checkpoint(pub usize);

impl Heap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.data.len())
    }

    pub fn undo(&mut self, checkpoint: Checkpoint) {
        self.data.truncate(checkpoint.0);
    }

    pub fn copy_clause_head(&mut self, clause: &HeapClausePtr) -> HeapTermPtr {
        let result = self.data.len();
        let offset = self.data.len() - clause.head();

        self.data.extend_from_within(clause.head()..clause.body());

        for term in &mut self.data[result..] {
            if let HeapTerm::Var(x, _, _, _) = term {
                *x += offset;
            }
        }

        result
    }

    pub fn copy_clause_body(&mut self, clause: &HeapClausePtr, choice_point_idx: ChoicePointIdx) {
        let result = self.data.len();
        let offset = self.data.len() - clause.body();

        self.data.extend_from_within(clause.body()..clause.end());

        for term in &mut self.data[result..] {
            match term {
                HeapTerm::Var(x, _, _, _) => *x += offset,
                HeapTerm::Cut(choice_point) => *choice_point += choice_point_idx,
                _ => {}
            }
        }
    }

    pub fn clause_goals<'a>(
        &'a mut self,
        clause: &HeapClausePtr,
    ) -> impl DoubleEndedIterator<Item = HeapTermPtr> + 'a {
        let offset = self.data.len() - clause.end();
        self.data[clause.goals()..clause.head()]
            .iter()
            .map(move |term| {
                if let HeapTerm::Var(x, _, _, _) = term {
                    x + offset
                } else {
                    unreachable!()
                }
            })
    }

    pub fn alloc(&mut self, term: HeapTerm) -> HeapTermPtr {
        let result = self.data.len();
        self.data.push(term);
        result
    }

    pub fn alloc_new_var(&mut self) -> HeapTermPtr {
        let result = self.data.len();
        self.data.push(HeapTerm::Var(result, false, false, 0));
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
                HeapTerm::Var(x, _, _, _) if *x != var => var = *x,
                _ => return var,
            }
        }
    }

    pub fn unify(&mut self, a: HeapTermPtr, b: HeapTermPtr) {
        match &mut self.data[a] {
            HeapTerm::Var(x, _, _, _) => *x = b,
            _ => unreachable!(),
        }
    }

    pub fn unbind(&mut self, term: HeapTermPtr) {
        match &mut self.data[term] {
            HeapTerm::Var(x, _, _, _) => *x = term,
            _ => unreachable!(),
        }
    }

    pub fn mark_shunted(&mut self, term: HeapTermPtr) {
        match &mut self.data[term] {
            HeapTerm::Var(_, shunted, _, _) => *shunted = true,
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
