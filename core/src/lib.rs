pub mod ast;
mod atom;
mod builtins;
mod goal;
mod stringmap;
mod trail;
mod vararena;

lalrpop_util::lalrpop_mod!(grammar);

#[cfg(test)]
mod tests;

use atom::Atom;
use goal::{GoalArena, GoalPtr};
use stringmap::StringMap;
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

pub trait ToCodeTerm {
    fn to_code_term(&self, string_map: &mut StringMap) -> CodeTerm;
}

pub struct WebPL {
    program: Program,
    string_map: StringMap,
}

pub struct Solver<'a> {
    program: &'a Program,
    goal: Option<GoalPtr>,
    goals: GoalArena,
    clause: usize,
    choice_points: Vec<ChoicePoint>,
    vars: VarArena<'a>,
    var_map: Vec<(String, HeapTermPtr)>,
    trail: Trail,
}

struct ChoicePoint {
    goal: Option<GoalPtr>,
    clause: usize,
    trail_checkpoint: trail::Checkpoint,
    arena_checkpoint: vararena::Checkpoint,
    goals_checkpoint: goal::Checkpoint,
}

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    BuiltinError(builtins::BuiltinError),
}

impl WebPL {
    pub fn new(program: impl AsRef<str>) -> Result<Self, Error> {
        let program = grammar::ProgramParser::new()
            .parse(program.as_ref())
            .map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(Self::from_ast(program))
    }

    pub fn from_ast(program: ast::Program) -> Self {
        let mut string_map = StringMap::default();

        let program = program
            .0
            .into_iter()
            .map(|clause| {
                (
                    clause.0.to_code_term(&mut string_map),
                    clause
                        .1
                        .into_iter()
                        .map(|term| term.to_code_term(&mut string_map))
                        .collect(),
                )
            })
            .collect();

        WebPL {
            program,
            string_map,
        }
    }

    pub fn solve(&mut self, query: impl AsRef<str>) -> Result<Solver, Error> {
        let query = grammar::QueryParser::new()
            .parse(query.as_ref())
            .map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(self.solve_from_ast(query))
    }

    pub fn solve_from_ast(&mut self, query: ast::Query) -> Solver {
        let query = query
            .0
            .into_iter()
            .map(|term| term.to_code_term(&mut self.string_map))
            .collect();

        Solver::solve(&self.program, &self.string_map, &query)
    }
}

impl<'a> Solver<'a> {
    pub fn solve(program: &'a Program, string_map: &'a StringMap, query: &Query) -> Self {
        let (vars, heap_query, var_map) = VarArena::new(string_map, query);
        let mut goals = GoalArena::default();

        let mut goal = None;

        for term in heap_query.into_iter().rev() {
            goal = Some(goals.alloc(term, goal));
        }

        Solver {
            program,
            goal,
            goals,
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
            let goal: HeapTermPtr = self.goals.get(*self.goal.as_ref()?).0;

            match builtins::eval(self, goal) {
                Some(Ok(true)) => {
                    // Built-in predicate succeeded
                    self.pop_goal();
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

                if self.pre_unify(goal, &clause.0) {
                    let head = self.vars.alloc(&clause.0, &mut var_map);

                    if self.unify(goal, head) {
                        self.push_choice_point(trail_checkpoint, arena_checkpoint);

                        self.clause = 0;
                        self.pop_goal();

                        clause.1.iter().rev().for_each(|goal| {
                            let heap_goal: HeapTermPtr = self.vars.alloc(goal, &mut var_map);
                            self.push_goal(heap_goal);
                        });

                        if let Some(solution) = self.succeed() {
                            self.pop_choice_point();
                            return Some(solution);
                        }

                        continue 'solve;
                    }
                }

                self.undo(trail_checkpoint, arena_checkpoint);

                self.clause += 1;
            }

            self.pop_choice_point()?;
        }
    }

    fn pre_unify(&self, a_ptr: HeapTermPtr, b: &CodeTerm) -> bool {
        match (self.vars.get(a_ptr), b) {
            (HeapTerm::Atom(a), CodeTerm::Atom(b)) => a == b,
            (HeapTerm::Var(_), _) | (_, CodeTerm::Var(_)) => true,
            (HeapTerm::Compound(f, a_arity, _), CodeTerm::Compound(g, b_args)) => {
                f == g && *a_arity == b_args.len()
            }
            _ => false,
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

    #[inline]
    fn push_goal(&mut self, goal: HeapTermPtr) {
        self.goal = Some(self.goals.alloc(goal, self.goal.take()));
    }

    #[inline]
    fn pop_goal(&mut self) {
        if let Some(goal) = self.goal.take() {
            self.goal = self.goals.get(goal).1;
        }
    }

    #[inline]
    fn succeed(&self) -> Option<Solution> {
        if self.goal.is_none() {
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
            goal: self.goal,
            clause: self.clause + 1,
            trail_checkpoint,
            arena_checkpoint,
            goals_checkpoint: self.goals.checkpoint(),
        });
    }

    #[inline]
    fn pop_choice_point(&mut self) -> Option<()> {
        if let Some(choice_point) = self.choice_points.pop() {
            self.goal = choice_point.goal;
            self.clause = choice_point.clause;
            self.undo(choice_point.trail_checkpoint, choice_point.arena_checkpoint);
            self.goals.undo(choice_point.goals_checkpoint);
            return Some(());
        }

        None
    }

    #[inline]
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
