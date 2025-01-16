use crate::atom::Atom;
use crate::stringmap::StringMap;
use crate::{ChoicePointIdx, ClauseName, CodeTerm, HeapTerm, HeapTermPtr, Query, StringId};

use std::fmt::Write;

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

    pub fn serialize(&self, terms: &[(String, usize)]) -> Vec<(String, String)> {
        let mut stacks = terms.iter().map(|(x, _)| (x.clone(), Vec::new())).collect();

        terms
            .iter()
            .enumerate()
            .map(|(i, (name, term))| {
                (
                    name.clone(),
                    self.serialize_inner(*term, Some(i), &mut stacks, false)
                        .unwrap(),
                )
            })
            .collect()
    }

    fn serialize_inner(
        &self,
        mut term: HeapTermPtr,
        stack: Option<usize>,
        stacks: &mut Vec<(String, Vec<HeapTermPtr>)>,
        mut continue_list: bool,
    ) -> Result<String, std::fmt::Error> {
        let mut result = String::new();

        loop {
            match &self.data[term] {
                HeapTerm::Atom(atom) => {
                    if continue_list {
                        if atom.is_nil() {
                            result.push(']');
                            return Ok(result);
                        }

                        result.push('|');
                    }

                    result.push_str(&atom.to_string(&self.string_map));
                }
                HeapTerm::Var(ptr) => {
                    if *ptr == term {
                        // The variable is unbound
                        if continue_list {
                            result.push('|');
                        }
                        write!(result, "_{}", ptr)?;
                    } else if let Some(name) = stacks
                        .iter()
                        .find(|(_, stack)| stack.contains(ptr))
                        .map(|(name, _)| name)
                    {
                        // We've found a cycle
                        if continue_list {
                            result.push('|');
                        }
                        result.push_str(name);
                    } else {
                        if let Some(stack) = stack {
                            // Keep track of this variable to find future cycles
                            stacks[stack].1.push(term);
                        }
                        term = *ptr;
                        continue;
                    }
                }
                HeapTerm::Compound(crate::stringmap::str::DOT, 2, next) => {
                    if let Some(HeapTerm::CompoundCons(head, tail)) = next.map(|x| &self.data[x]) {
                        let element = self.serialize_inner(*head, None, stacks, false)?;

                        if element == "]" {
                            match continue_list {
                                true => write!(result, "]")?,
                                false => write!(result, "[]")?,
                            };
                            return Ok(result);
                        }

                        if let Some(HeapTerm::CompoundCons(next_term, None)) =
                            tail.map(|x| &self.data[x])
                        {
                            match continue_list {
                                true => write!(result, ",{}", element)?,
                                false => write!(result, "[{}", element)?,
                            };

                            term = *next_term;
                            continue_list = true;
                            continue;
                        }
                    }

                    unreachable!();
                }
                HeapTerm::Compound(functor, arity, next) => {
                    if continue_list {
                        result.push('|');
                    }

                    write!(result, "{}(", self.get_atom(*functor))?;

                    let mut arg = *next;
                    for i in 0..*arity {
                        if let Some(HeapTerm::CompoundCons(head, tail)) = arg.map(|x| &self.data[x])
                        {
                            result.push_str(&self.serialize_inner(*head, None, stacks, false)?);

                            if i + 1 < *arity {
                                result.push(',');
                            }
                            arg = *tail;
                        } else {
                            unreachable!();
                        }
                    }

                    result.push(')');
                }
                HeapTerm::CompoundCons(_, _) => unreachable!(),
                HeapTerm::Cut(_) => result.push('!'),
            };

            if continue_list {
                result.push(']');
            }

            return Ok(result);
        }
    }
}
