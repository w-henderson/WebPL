pub mod ast;
mod atom;
mod builtins;
mod compile;
mod goal;
mod heap;
mod stringmap;
mod trail;
mod wasm;

pub use wasm::*;

lalrpop_util::lalrpop_mod!(grammar);

#[cfg(test)]
mod tests;

use atom::Atom;
use compile::compile;
use goal::Goals;
use heap::Heap;
use stringmap::StringMap;
use trail::Trail;

type HeapTermPtr = usize;
type ChoicePointIdx = usize;

type StringId = usize;

pub enum HeapTerm {
    Atom(Atom),
    Var(HeapTermPtr),
    Compound(StringId, usize, Option<HeapTermPtr>),
    CompoundCons(HeapTermPtr, Option<HeapTermPtr>),
    Cut(ChoicePointIdx),
}

pub enum CodeTerm {
    Atom(Atom),
    Var(StringId),
    Compound(StringId, Vec<CodeTerm>),
    Cut,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct ClauseName(pub StringId, pub usize); // functor, arity

pub struct Clause {
    head: CodeTerm,
    body: Vec<CodeTerm>,
}

pub type Query = Vec<CodeTerm>;
pub type Program = Vec<(ClauseName, Vec<Clause>)>;
pub type Solution = Vec<(String, String)>;

pub struct Solver {
    program: Program,
    goals: Goals,
    group: Option<usize>,
    clause: usize,
    choice_points: Vec<ChoicePoint>,
    heap: Heap,
    var_map: Vec<(String, HeapTermPtr)>,
    trail: Trail,
}

struct ChoicePoint {
    group: Option<usize>,
    clause: usize,
    trail_checkpoint: trail::Checkpoint,
    heap_checkpoint: heap::Checkpoint,
    goals_checkpoint: goal::Checkpoint,
}

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    BuiltinError(builtins::BuiltinError),
}

impl Solver {
    pub fn new(program: impl AsRef<str>, query: impl AsRef<str>) -> Result<Self, Error> {
        let program = grammar::ProgramParser::new()
            .parse(program.as_ref())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        let query = grammar::QueryParser::new()
            .parse(query.as_ref())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        Ok(Self::from_ast(program, query))
    }

    pub fn from_ast(program: ast::Program, query: ast::Query) -> Self {
        let mut string_map = StringMap::default();

        let program = compile(program, &mut string_map);

        let query = query
            .0
            .into_iter()
            .map(|term| term.to_code_term(&mut string_map).0)
            .collect();

        let (vars, heap_query, var_map) = Heap::new(string_map, &query);
        let goals = Goals::new(&heap_query);

        let mut solver = Solver {
            program,
            goals,
            group: None,
            clause: 0,
            choice_points: Vec::new(),
            heap: vars,
            var_map,
            trail: Trail::new(),
        };

        solver.find_clause_group();

        solver
    }

    fn step(&mut self) -> Option<Solution> {
        let mut var_map = Vec::new();

        'solve: loop {
            let goal: HeapTermPtr = self.goals.current()?;

            match builtins::eval(self, goal) {
                Some(Ok(true)) => {
                    // Built-in predicate succeeded
                    self.goals.pop();
                    self.find_clause_group();
                    if self.goals.is_complete() {
                        let solution = self.serialize_solution();
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

            let group = self.group?;

            while self.clause < self.program[group].1.len() {
                var_map.clear();

                let head = &self.program[group].1[self.clause].head;

                if self.pre_unify(goal, head) {
                    let choice_point = self.enter();

                    let head = self
                        .heap
                        .alloc(head, &mut var_map, self.choice_points.len());

                    if self.unify(goal, head) {
                        // If this was the only choice, don't push a choice point
                        if self.clause + 1 < self.program[group].1.len() {
                            self.choice_points.push(choice_point);
                        }

                        self.goals.pop();
                        let body = &self.program[group].1[self.clause].body;
                        for goal in body.iter().rev() {
                            self.goals.push(self.heap.alloc(
                                goal,
                                &mut var_map,
                                self.choice_points.len().saturating_sub(1),
                            ));
                        }

                        self.find_clause_group();

                        if self.goals.is_complete() {
                            let solution = self.serialize_solution();
                            self.pop_choice_point();
                            return Some(solution);
                        }

                        continue 'solve;
                    }

                    self.undo(choice_point);
                }
            }

            self.pop_choice_point()?;
        }
    }

    fn pre_unify(&self, a_ptr: HeapTermPtr, b: &CodeTerm) -> bool {
        match (self.heap.get(a_ptr), b) {
            (HeapTerm::Atom(a), CodeTerm::Atom(b)) => a == b,
            (HeapTerm::Var(_), _) | (_, CodeTerm::Var(_)) => true,
            (HeapTerm::Compound(f, a_arity, _), CodeTerm::Compound(g, b_args)) => {
                f == g && *a_arity == b_args.len()
            }
            _ => false,
        }
    }

    fn unify(&mut self, a_ptr: HeapTermPtr, b_ptr: HeapTermPtr) -> bool {
        match (self.heap.get(a_ptr), self.heap.get(b_ptr)) {
            (HeapTerm::Atom(a), HeapTerm::Atom(b)) => a == b,
            (HeapTerm::Var(a), _) => {
                self.trail.push(*a);
                self.heap.unify(*a, b_ptr);
                true
            }
            (_, HeapTerm::Var(b)) => {
                self.trail.push(*b);
                self.heap.unify(*b, a_ptr);
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
                    ) = (self.heap.get(a_ref), self.heap.get(b_ref))
                    {
                        a_arg = *a_tail;
                        b_arg = *b_tail;

                        if !self.unify(*a_head, *b_head) {
                            self.trail.undo(checkpoint, &mut self.heap);
                            return false;
                        }
                    };
                }

                true
            }
            _ => false,
        }
    }

    #[inline]
    fn enter(&self) -> ChoicePoint {
        ChoicePoint {
            group: self.group,
            clause: self.clause + 1,
            trail_checkpoint: self.trail.checkpoint(),
            heap_checkpoint: self.heap.checkpoint(),
            goals_checkpoint: self.goals.checkpoint(),
        }
    }

    #[inline]
    fn undo(&mut self, choice_point: ChoicePoint) {
        self.group = choice_point.group;
        self.clause = choice_point.clause;
        self.trail
            .undo(choice_point.trail_checkpoint, &mut self.heap);
        self.heap.undo(choice_point.heap_checkpoint);
        self.goals.undo(choice_point.goals_checkpoint);
    }

    #[inline]
    fn pop_choice_point(&mut self) -> Option<()> {
        self.choice_points
            .pop()
            .map(|choice_point| self.undo(choice_point))
    }

    #[inline]
    fn find_clause_group(&mut self) {
        if let Some(goal) = self.goals.current() {
            let name = self.heap.get_name(goal);
            self.group = self
                .program
                .iter()
                .position(|(clause_name, _)| clause_name == &name);
            self.clause = 0;
        }
    }

    #[inline]
    fn cut(&mut self, choice_point_idx: ChoicePointIdx) {
        self.choice_points.truncate(choice_point_idx);
    }

    #[inline]
    fn serialize_solution(&self) -> Solution {
        self.var_map
            .iter()
            .map(|(name, ptr)| (name.clone(), self.heap.serialize(*ptr, name)))
            .collect::<Vec<_>>()
    }

    #[cfg(test)]
    pub(crate) fn max_choice_point_stack_height(&self) -> usize {
        self.choice_points.capacity()
    }
}

impl Iterator for Solver {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.step()
    }
}
