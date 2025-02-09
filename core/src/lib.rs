pub mod ast;
mod atom;
mod builtins;
mod compile;
mod gc;
mod goal;
mod heap;
mod serialize;
mod stringmap;
mod trail;
mod wasm;

pub use wasm::*;

lalrpop_util::lalrpop_mod!(grammar);

use serde::Serialize;

#[cfg(test)]
mod tests;

use atom::Atom;
use compile::compile;
use gc::{GCRewritable, GarbageCollector};
use goal::Goals;
use heap::Heap;
use stringmap::StringMap;
use trail::Trail;

type HeapTermPtr = usize;
type ChoicePointIdx = usize;

type StringId = usize;

static GC_HEAP_SIZE_THRESHOLD: usize = 1024;
static GC_HEAP_PRESSURE_THRESHOLD: f64 = 0.9;
static GC_COOLDOWN: usize = 16;

#[derive(Clone, Copy, Debug)]
pub enum HeapTerm {
    Atom(Atom),
    Var(HeapTermPtr, bool), // ptr, shunted
    Compound(StringId, usize),
    Cut(ChoicePointIdx),
    Lambda(StringId, usize),
}

pub enum CodeTerm {
    Atom(Atom),
    Var(StringId),
    Compound(StringId, Vec<CodeTerm>),
    Cut,
    Lambda(StringId, Vec<CodeTerm>),
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
    choice_point_age: heap::Checkpoint,
    heap: Heap,
    gc: GarbageCollector,
    var_map: Vec<(String, HeapTermPtr)>,
    trail: Trail,

    #[cfg(test)]
    interrupt: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Copy, Clone)]
struct ChoicePoint {
    group: Option<usize>,
    clause: usize,
    trail_checkpoint: trail::Checkpoint,
    heap_checkpoint: heap::Checkpoint,
    goals_checkpoint: goal::Checkpoint,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Error {
    pub location: Option<ErrorLocation>,
    pub error: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ErrorLocation {
    pub query: bool,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Solver {
    pub fn new(program: impl AsRef<str>, query: impl AsRef<str>) -> Result<Self, Error> {
        let (program, query) = Self::parse(program, query)?;
        Ok(Self::from_ast(program, query, false))
    }

    pub fn new_with_gc(program: impl AsRef<str>, query: impl AsRef<str>) -> Result<Self, Error> {
        let (program, query) = Self::parse(program, query)?;
        Ok(Self::from_ast(program, query, true))
    }

    pub fn parse(
        program: impl AsRef<str>,
        query: impl AsRef<str>,
    ) -> Result<(ast::Program, ast::Query), Error> {
        let program = grammar::ProgramParser::new()
            .parse(program.as_ref())
            .map_err(|e| ast::parse_error(program.as_ref(), false, e))?;

        let query = grammar::QueryParser::new()
            .parse(query.as_ref())
            .map_err(|e| ast::parse_error(query.as_ref(), true, e))?;

        Ok((program, query))
    }

    pub fn from_ast(program: ast::Program, query: ast::Query, gc: bool) -> Self {
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
            choice_point_age: heap::Checkpoint(0),
            heap: vars,
            gc: if gc {
                GarbageCollector::new(
                    GC_HEAP_SIZE_THRESHOLD,
                    GC_HEAP_PRESSURE_THRESHOLD,
                    GC_COOLDOWN,
                )
            } else {
                GarbageCollector::disabled()
            },
            var_map,
            trail: Trail::new(),

            #[cfg(test)]
            interrupt: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        solver.find_clause_group();

        solver
    }

    fn step(&mut self) -> Result<Option<Solution>, Error> {
        self.step_inner().transpose()
    }

    fn step_inner(&mut self) -> Option<Result<Solution, Error>> {
        let mut var_map = Vec::new();

        'solve: loop {
            #[cfg(test)]
            self.check_interrupted()?;

            if self.gc.pre_run(&self.heap, self.choice_points.len()) {
                GarbageCollector::run(self);
            }

            let goal: HeapTermPtr = self.goals.current()?;

            match builtins::eval(self, goal) {
                Some(Ok(true)) => {
                    // Built-in predicate succeeded
                    self.goals.pop();
                    self.find_clause_group();
                    if self.goals.is_complete() {
                        let solution = self.serialize_solution();
                        self.pop_choice_point();
                        return Some(Ok(solution));
                    }
                    continue;
                }
                Some(Ok(false)) => {
                    // Built-in predicate failed
                    self.pop_choice_point()?;
                    continue;
                }
                Some(Err(e)) => return Some(Err(builtins::error(self, e))), // Built-in predicate had an error
                None => {} // This goal is not a built-in predicate
            };

            if let Some(group) = self.group {
                while self.clause < self.program[group].1.len() {
                    var_map.clear();

                    let head = &self.program[group].1[self.clause].head;

                    if self.pre_unify(goal, head) {
                        let choice_point = self.enter();
                        let choice_point_idx = self.choice_points.len();

                        if self.clause + 1 < self.program[group].1.len() {
                            self.choice_point_age = choice_point.heap_checkpoint;
                        }

                        let head = self.heap.alloc(head, &mut var_map, choice_point_idx);

                        if self.unify(goal, head) {
                            // If this was the only choice, don't push a choice point
                            if self.clause + 1 < self.program[group].1.len() {
                                self.push_choice_point(choice_point);
                            }

                            self.goals.pop();
                            let body = &self.program[group].1[self.clause].body;
                            for goal in body.iter().rev() {
                                self.goals.push(self.heap.alloc(
                                    goal,
                                    &mut var_map,
                                    choice_point_idx,
                                ));
                            }

                            self.find_clause_group();

                            if self.goals.is_complete() {
                                let solution = self.serialize_solution();
                                self.pop_choice_point();
                                return Some(Ok(solution));
                            }

                            continue 'solve;
                        }

                        self.undo(choice_point);
                    }
                }
            }

            self.pop_choice_point()?;
        }
    }

    fn pre_unify(&self, a_ptr: HeapTermPtr, b: &CodeTerm) -> bool {
        match (self.heap.get(a_ptr), b) {
            (HeapTerm::Atom(a), CodeTerm::Atom(b)) => a == b,
            (HeapTerm::Var(_, _), _) | (_, CodeTerm::Var(_)) => true,
            (HeapTerm::Compound(f, a_arity), CodeTerm::Compound(g, b_args)) => {
                f == g && *a_arity == b_args.len()
            }
            _ => false,
        }
    }

    fn unify(&mut self, a_ptr: HeapTermPtr, b_ptr: HeapTermPtr) -> bool {
        let a_root = self.heap.get_ptr(a_ptr);
        let b_root = self.heap.get_ptr(b_ptr);

        match (self.heap.get(a_root), self.heap.get(b_root)) {
            (HeapTerm::Atom(a), HeapTerm::Atom(b)) => a == b,

            // Unify variables downwards (i.e. newer variables point to older ones)
            (HeapTerm::Var(a, _), HeapTerm::Var(b, _)) if *a < b_root => self.unify_var(*b, a_root),
            (HeapTerm::Var(a, _), _) => self.unify_var(*a, b_root),
            (_, HeapTerm::Var(b, _)) => self.unify_var(*b, a_root),

            (HeapTerm::Compound(f, a_arity), HeapTerm::Compound(g, b_arity)) => {
                if f != g || a_arity != b_arity {
                    return false;
                }

                let checkpoint = self.trail.checkpoint();

                for (a_arg, b_arg) in (1..=*a_arity).map(|i| (a_root + i, b_root + i)) {
                    if !self.unify(a_arg, b_arg) {
                        self.trail.undo(checkpoint, &mut self.heap);
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    #[inline]
    fn unify_var(&mut self, a: HeapTermPtr, b: HeapTermPtr) -> bool {
        if a < self.choice_point_age.0 {
            self.trail.push(a);
        } else {
            self.heap.mark_shunted(a);
        }

        self.heap.unify(a, b);
        true
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
    fn push_choice_point(&mut self, choice_point: ChoicePoint) {
        self.choice_point_age = choice_point.heap_checkpoint;
        self.choice_points.push(choice_point);
    }

    #[inline]
    fn pop_choice_point(&mut self) -> Option<()> {
        self.choice_points.pop().map(|choice_point| {
            self.choice_point_age = self
                .choice_points
                .last()
                .map(|cp| cp.heap_checkpoint)
                .unwrap_or(heap::Checkpoint(0));
            self.undo(choice_point)
        })
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
    pub(crate) fn serialize_solution(&mut self) -> Solution {
        self.heap.serialize(&self.var_map)
    }
}

impl Iterator for Solver {
    type Item = Result<Solution, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.step_inner()
    }
}

impl GCRewritable for [ChoicePoint] {
    fn rewrite(&mut self, from: usize, map: &[usize], trail_map: &[usize], goal_map: &[usize]) {
        for cp in self.iter_mut().skip(from) {
            cp.heap_checkpoint = crate::heap::Checkpoint(map[cp.heap_checkpoint.0]);
            cp.trail_checkpoint = crate::trail::Checkpoint(trail_map[cp.trail_checkpoint.0]);

            if let Some(goal) = cp.goals_checkpoint.0 {
                cp.goals_checkpoint =
                    crate::goal::Checkpoint(Some(goal_map[goal]), goal_map[cp.goals_checkpoint.1]);
            } else {
                cp.goals_checkpoint =
                    crate::goal::Checkpoint(None, goal_map[cp.goals_checkpoint.1]);
            }
        }
    }
}
