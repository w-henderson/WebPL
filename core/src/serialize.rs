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
            .filter(|(x, _)| x != "_")
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
                HeapTerm::Var(ptr, _, _, _) => {
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
                            stacks[stack].1.push(*ptr);
                        }
                        term = *ptr;
                        continue;
                    }
                }
                HeapTerm::Compound(crate::stringmap::str::DOT, 2) => {
                    let element = self.serialize_inner(term + 1, None, stacks, false)?;

                    if element == "]" {
                        match continue_list {
                            true => write!(result, "]")?,
                            false => write!(result, "[]")?,
                        };
                        return Ok(result);
                    }

                    match continue_list {
                        true => write!(result, ",{}", element)?,
                        false => write!(result, "[{}", element)?,
                    };

                    term += 2;
                    continue_list = true;
                    continue;
                }
                HeapTerm::Compound(functor, arity) => {
                    if continue_list {
                        result.push('|');
                    }

                    write!(result, "{}(", self.get_atom(*functor))?;

                    for i in 1..=*arity {
                        result.push_str(&self.serialize_inner(term + i, None, stacks, false)?);

                        if i < *arity {
                            result.push(',');
                        }
                    }

                    result.push(')');
                }
                HeapTerm::Cut(_) => result.push('!'),
                HeapTerm::Lambda(_, _) => result.push_str("<js_function>"),
            };

            if continue_list {
                result.push(']');
            }

            return Ok(result);
        }
    }
}
