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
use gc::{GCRewritable, GarbageCollector};
use goal::Goals;
use heap::Heap;
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

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct ClauseName(pub StringId, pub usize); // functor, arity

pub struct HeapClausePtr {
    ptr: HeapTermPtr,
    goals_length: usize,
    head_length: usize,
    body_length: usize,
}

pub type Index = Vec<(ClauseName, Vec<HeapClausePtr>)>;

pub type Solution = Vec<(String, String)>;

pub struct Solver {
    index: Index,
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
        let mut heap = Heap::new();

        let index = compile::compile(program, &mut heap);
        let (query, var_map) = compile::alloc_query(query, &mut heap);
        let goals = Goals::new(&query);

        let mut solver = Solver {
            index,
            goals,
            group: None,
            clause: 0,
            choice_points: Vec::new(),
            choice_point_age: heap::Checkpoint(0),
            heap,
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
                    self.goals.pop(true);
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
                while self.clause < self.index[group].1.len() {
                    let choice_point = self.enter();
                    let choice_point_idx = self.choice_points.len();
                    self.choice_point_age = choice_point.heap_checkpoint;

                    let head = self
                        .heap
                        .copy_clause_head(&self.index[group].1[self.clause]);

                    if self.unify(goal, head) {
                        // If this was the only choice, don't push a choice point
                        let determinate = if self.clause + 1 < self.index[group].1.len() {
                            self.push_choice_point(choice_point);
                            false
                        } else {
                            true
                        };

                        let clause = &self.index[group].1[self.clause];

                        self.goals.pop(determinate);
                        self.heap.copy_clause_body(clause, choice_point_idx);

                        for goal in self.heap.clause_goals(clause).rev() {
                            self.goals.push(goal);
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

            self.pop_choice_point()?;
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
                .index
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
    fn rewrite(&mut self, from: usize, map: &[usize], trail_map: &[usize]) {
        for cp in self.iter_mut().skip(from) {
            cp.heap_checkpoint = crate::heap::Checkpoint(map[cp.heap_checkpoint.0]);
            cp.trail_checkpoint = crate::trail::Checkpoint(trail_map[cp.trail_checkpoint.0]);
        }
    }
}

impl HeapClausePtr {
    #[inline]
    fn goals(&self) -> HeapTermPtr {
        self.ptr
    }

    #[inline]
    fn head(&self) -> HeapTermPtr {
        self.ptr + self.goals_length
    }

    #[inline]
    fn body(&self) -> HeapTermPtr {
        self.ptr + self.goals_length + self.head_length
    }

    #[inline]
    fn end(&self) -> HeapTermPtr {
        self.ptr + self.goals_length + self.head_length + self.body_length
    }
}
