mod builtins;
mod trail;
mod vararena;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use trail::Trail;
use vararena::VarArena;

type Atom = String;
type VarName = String;
type HeapTermPtr = usize;

pub enum HeapTerm {
    Atom(Atom),
    Var(HeapTermPtr),
    Compound(Atom, Vec<HeapTermPtr>),
}

pub enum CodeTerm {
    Atom(Atom),
    Var(VarName),
    Compound(Atom, Vec<CodeTerm>),
}

pub type Clause = (CodeTerm, Vec<CodeTerm>);
pub type Query = Vec<CodeTerm>;
pub type Program = Vec<Clause>;
pub type Solution = Vec<(VarName, String)>;

pub struct Solver<'a> {
    program: &'a Program,
    goals: VecDeque<HeapTermPtr>,
    vars: VarArena,
    var_map: Vec<(VarName, HeapTermPtr)>,
    trail: Trail,
}

impl<'a> Solver<'a> {
    pub fn solve(program: &'a Program, query: &Query) -> Self {
        let (vars, heap_query, var_map) = VarArena::new(query);

        Solver {
            program,
            goals: heap_query.clone().into(),
            vars,
            var_map,
            trail: Trail::new(),
        }
    }

    fn step(&mut self, depth: usize) -> Option<Solution> {
        let goal: HeapTermPtr = self.goals.pop_front()?;

        match builtins::eval(self, goal) {
            Some(Ok(true)) => return self.succeed(depth), // Built-in predicate succeeded
            Some(Ok(false)) => return self.fail(depth),   // Built-in predicate failed
            Some(Err(())) => panic!("Error"),             // Built-in predicate had an error
            None => {}                                    // This goal is not a built-in predicate
        };

        for clause in self.program {
            let (trail_checkpoint, arena_checkpoint) = self.enter();

            let (head, body) = self.vars.alloc_clause(clause);

            if self.unify(goal, head) {
                body.iter()
                    .rev()
                    .for_each(|goal| self.goals.push_front(*goal));

                if let Some(solution) = self.succeed(depth) {
                    return Some(solution);
                }
            }

            self.undo(trail_checkpoint, arena_checkpoint);
        }

        self.fail(depth)
    }

    fn unify(&mut self, a_ptr: HeapTermPtr, b_ptr: HeapTermPtr) -> bool {
        match (self.vars.get(a_ptr), self.vars.get(b_ptr)) {
            (HeapTerm::Atom(a), HeapTerm::Atom(b)) => a == b,
            (HeapTerm::Var(a), _) => {
                self.trail.push(*a);
                self.vars.unify(*a, b_ptr);
                true
            }
            (_, HeapTerm::Var(b)) => {
                self.trail.push(*b);
                self.vars.unify(*b, a_ptr);
                true
            }
            (HeapTerm::Compound(f, a_args), HeapTerm::Compound(g, b_args)) => {
                if f != g || a_args.len() != b_args.len() {
                    return false;
                }

                let checkpoint = self.trail.checkpoint();

                for (a, b) in a_args.clone().into_iter().zip(b_args.clone().into_iter()) {
                    if !self.unify(a, b) {
                        self.trail.undo(checkpoint, &mut self.vars);
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    fn serialize_solution(&self) -> Solution {
        self.var_map
            .iter()
            .map(|(name, var)| (name.clone(), self.vars.serialize(*var, name)))
            .collect::<Vec<_>>()
    }

    #[inline]
    fn enter(&mut self) -> (trail::Checkpoint, vararena::Checkpoint) {
        (self.trail.checkpoint(), self.vars.checkpoint())
    }

    #[inline]
    fn succeed(&mut self, depth: usize) -> Option<Solution> {
        if self.goals.is_empty() {
            Some(self.serialize_solution())
        } else {
            self.step(depth + 1)
        }
    }

    #[inline]
    fn undo(
        &mut self,
        trail_checkpoint: trail::Checkpoint,
        arena_checkpoint: vararena::Checkpoint,
    ) {
        self.trail.undo(trail_checkpoint, &mut self.vars);
        self.vars.undo(arena_checkpoint);
    }

    #[inline]
    fn fail(&mut self, depth: usize) -> Option<Solution> {
        self.step(depth + 1)
    }
}

impl Iterator for Solver<'_> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.step(0)
    }
}
