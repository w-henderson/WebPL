use crate::heap::Heap;
use crate::{HeapTerm, HeapTermPtr};

use std::fmt::Write;

impl Heap {
    pub fn serialize(&self, terms: &[(String, usize)]) -> Vec<(String, String)> {
        let mut stacks: Vec<(String, Vec<HeapTermPtr>)> = terms
            .iter()
            .map(|(x, ptr)| (x.clone(), vec![*ptr]))
            .collect();

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
                HeapTerm::Var(ptr, _) => {
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
