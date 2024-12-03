pub mod ast;
mod atom;
mod builtins;
mod goal;
mod heap;
mod stringmap;
mod trail;

lalrpop_util::lalrpop_mod!(grammar);

#[cfg(test)]
mod tests;

use atom::Atom;
use goal::Goals;
use heap::Heap;
use stringmap::StringMap;
use trail::Trail;

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

pub struct Solver {
    program: Program,
    goals: Goals,
    clause: usize,
    choice_points: Vec<ChoicePoint>,
    heap: Heap,
    var_map: Vec<(String, HeapTermPtr)>,
    trail: Trail,
}

struct ChoicePoint {
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

        let query = query
            .0
            .into_iter()
            .map(|term| term.to_code_term(&mut string_map))
            .collect();

        let (vars, heap_query, var_map) = Heap::new(string_map, &query);
        let goals = Goals::new(&heap_query);

        Solver {
            program,
            goals,
            clause: 0,
            choice_points: Vec::new(),
            heap: vars,
            var_map,
            trail: Trail::new(),
        }
    }

    fn step(&mut self) -> Option<Solution> {
        let mut var_map = Vec::new();

        'solve: loop {
            let goal: HeapTermPtr = self.goals.current()?;

            match builtins::eval(self, goal) {
                Some(Ok(true)) => {
                    // Built-in predicate succeeded
                    self.goals.pop();
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

            while self.clause < self.program.len() {
                let (trail_checkpoint, heap_checkpoint) = self.enter();

                var_map.clear();

                let (head, _) = &self.program[self.clause];

                if self.pre_unify(goal, head) {
                    let head = self.heap.alloc(head, &mut var_map);

                    if self.unify(goal, head) {
                        self.push_choice_point(trail_checkpoint, heap_checkpoint);

                        self.goals.pop();
                        let (_, body) = &self.program[self.clause];
                        for goal in body.iter().rev() {
                            self.goals.push(self.heap.alloc(goal, &mut var_map));
                        }

                        self.clause = 0;

                        if self.goals.is_complete() {
                            let solution = self.serialize_solution();
                            self.pop_choice_point();
                            return Some(solution);
                        }

                        continue 'solve;
                    }
                }

                self.undo(trail_checkpoint, heap_checkpoint);

                self.clause += 1;
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
    fn push_choice_point(
        &mut self,
        trail_checkpoint: trail::Checkpoint,
        heap_checkpoint: heap::Checkpoint,
    ) {
        self.choice_points.push(ChoicePoint {
            clause: self.clause + 1,
            trail_checkpoint,
            heap_checkpoint,
            goals_checkpoint: self.goals.checkpoint(),
        });
    }

    #[inline]
    fn pop_choice_point(&mut self) -> Option<()> {
        if let Some(choice_point) = self.choice_points.pop() {
            self.clause = choice_point.clause;
            self.undo(choice_point.trail_checkpoint, choice_point.heap_checkpoint);
            self.goals.undo(choice_point.goals_checkpoint);
            return Some(());
        }

        None
    }

    #[inline]
    fn serialize_solution(&self) -> Solution {
        self.var_map
            .iter()
            .map(|(name, ptr)| (name.clone(), self.heap.serialize(*ptr, name)))
            .collect::<Vec<_>>()
    }

    #[inline]
    fn enter(&mut self) -> (trail::Checkpoint, heap::Checkpoint) {
        (self.trail.checkpoint(), self.heap.checkpoint())
    }

    #[inline]
    fn undo(&mut self, trail_checkpoint: trail::Checkpoint, heap_checkpoint: heap::Checkpoint) {
        self.trail.undo(trail_checkpoint, &mut self.heap);
        self.heap.undo(heap_checkpoint);
    }
}

impl Iterator for Solver {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.step()
    }
}
