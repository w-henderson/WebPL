pub mod ast;
mod atom;
mod builtins;
mod trail;
mod vararena;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use atom::Atom;
use trail::Trail;
use vararena::VarArena;

type HeapTermPtr = usize;

type StringId = usize;

pub enum HeapTerm {
    Atom(Atom),
    Var(HeapTermPtr),
    Compound(StringId, usize, Option<HeapTermPtr>),
    CompoundCons(HeapTermPtr, Option<HeapTermPtr>),
}

pub enum CodeTerm {
    Atom(Atom),
    Var(StringId),
    Compound(StringId, Vec<CodeTerm>),
}

pub type Clause = (CodeTerm, Vec<CodeTerm>);
pub type Query = Vec<CodeTerm>;
pub type Program = Vec<Clause>;
pub type Solution = Vec<(String, String)>;

pub struct Solver<'a> {
    program: &'a Program,
    goals: VecDeque<HeapTermPtr>,
    clause: usize,
    choice_points: Vec<ChoicePoint>,
    vars: VarArena,
    var_map: Vec<(String, HeapTermPtr)>,
    trail: Trail,
}

struct ChoicePoint {
    goals: VecDeque<HeapTermPtr>,
    clause: usize,
    trail_checkpoint: trail::Checkpoint,
    arena_checkpoint: vararena::Checkpoint,
}

impl<'a> Solver<'a> {
    pub fn solve(program: &'a Program, string_map: ast::StringMap, query: &Query) -> Self {
        let (vars, heap_query, var_map) = VarArena::new(string_map, query);

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
        let mut var_map = Vec::new();

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

                var_map.clear();

                let head = self.vars.alloc(&clause.0, &mut var_map);

                if self.unify(goal, head) {
                    self.push_choice_point(trail_checkpoint, arena_checkpoint);

                    self.clause = 0;
                    self.goals.pop_front();

                    clause.1.iter().rev().for_each(|goal| {
                        self.goals.push_front(self.vars.alloc(goal, &mut var_map));
                    });

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
            (HeapTerm::Compound(f, a_arity, a_next), HeapTerm::Compound(g, b_arity, b_next)) => {
                if f != g || a_arity != b_arity {
                    return false;
                }

                let checkpoint = self.trail.checkpoint();

                let mut a_arg = *a_next;
                let mut b_arg = *b_next;

                while let Some((a_ref, b_ref)) = a_arg.zip(b_arg) {
                    if let (
                        HeapTerm::CompoundCons(a_head, a_tail),
                        HeapTerm::CompoundCons(b_head, b_tail),
                    ) = (self.vars.get(a_ref), self.vars.get(b_ref))
                    {
                        a_arg = *a_tail;
                        b_arg = *b_tail;

                        if !self.unify(*a_head, *b_head) {
                            self.trail.undo(checkpoint, &mut self.vars);
                            return false;
                        }
                    };
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
            .map(|(name, ptr)| (name.clone(), self.vars.serialize(*ptr, name)))
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
