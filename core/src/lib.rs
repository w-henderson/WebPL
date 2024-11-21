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
    clause: usize,
    choice_points: Vec<ChoicePoint>,
    vars: VarArena,
    var_map: Vec<(VarName, HeapTermPtr)>,
    trail: Trail,
}

struct ChoicePoint {
    goals: VecDeque<HeapTermPtr>,
    clause: usize,
    trail_checkpoint: trail::Checkpoint,
    arena_checkpoint: vararena::Checkpoint,
}

impl<'a> Solver<'a> {
    pub fn solve(program: &'a Program, query: &Query) -> Self {
        let (vars, heap_query, var_map) = VarArena::new(query);

        Solver {
            program,
            goals: heap_query.clone().into(),
            clause: 0,
            choice_points: Vec::new(),
            vars,
            var_map,
            trail: Trail::new(),
        }
    }

    fn step(&mut self) -> Option<Solution> {
        'solve: loop {
            let goal: HeapTermPtr = *self.goals.front()?;

            match builtins::eval(self, goal) {
                Some(Ok(true)) => {
                    // Built-in predicate succeeded
                    self.goals.pop_front();
                    if let Some(solution) = self.succeed() {
                        self.pop_choice_point();
                        return Some(solution);
                    }
                    continue;
                }
                Some(Ok(false)) => {
                    // Built-in predicate failed
                    self.pop_choice_point()?;
                    continue;
                }
                Some(Err(e)) => panic!("Error: {:?}", e), // Built-in predicate had an error
                None => {}                                // This goal is not a built-in predicate
            };

            while self.clause < self.program.len() {
                let clause = &self.program[self.clause];

                let (trail_checkpoint, arena_checkpoint) = self.enter();

                let (head, body) = self.vars.alloc_clause(clause);

                if self.unify(goal, head) {
                    self.push_choice_point(trail_checkpoint, arena_checkpoint);

                    self.clause = 0;
                    self.goals.pop_front();

                    body.iter()
                        .rev()
                        .for_each(|goal| self.goals.push_front(*goal));

                    if let Some(solution) = self.succeed() {
                        self.pop_choice_point();
                        return Some(solution);
                    }

                    continue 'solve;
                }

                self.undo(trail_checkpoint, arena_checkpoint);

                self.clause += 1;
            }

            self.pop_choice_point()?;
        }
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

    fn succeed(&self) -> Option<Solution> {
        if self.goals.is_empty() {
            return Some(self.serialize_solution());
        }

        None
    }

    #[inline]
    fn push_choice_point(
        &mut self,
        trail_checkpoint: trail::Checkpoint,
        arena_checkpoint: vararena::Checkpoint,
    ) {
        self.choice_points.push(ChoicePoint {
            goals: self.goals.clone(),
            clause: self.clause + 1,
            trail_checkpoint,
            arena_checkpoint,
        });
    }

    #[inline]
    fn pop_choice_point(&mut self) -> Option<()> {
        if let Some(choice_point) = self.choice_points.pop() {
            self.goals = choice_point.goals;
            self.clause = choice_point.clause;
            self.undo(choice_point.trail_checkpoint, choice_point.arena_checkpoint);
            return Some(());
        }

        None
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
    fn undo(
        &mut self,
        trail_checkpoint: trail::Checkpoint,
        arena_checkpoint: vararena::Checkpoint,
    ) {
        self.trail.undo(trail_checkpoint, &mut self.vars);
        self.vars.undo(arena_checkpoint);
    }
}

impl Iterator for Solver<'_> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.step()
    }
}
