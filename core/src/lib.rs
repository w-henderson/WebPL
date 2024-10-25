mod trail;
mod vararena;

use std::collections::VecDeque;

use trail::Trail;
use vararena::VarArena;

type Atom = String;
type VarName = String;
type HeapTermPtr = usize;

pub enum HeapTerm {
    Atom(Atom),
    Var(HeapTermPtr),
    Compound(Atom, usize), // (functor, arity)
}

pub enum CodeTerm {
    Atom(Atom),
    Var(VarName),
    Compound(Atom, Vec<CodeTerm>), // (functor, args)
}

pub type Clause = (CodeTerm, Vec<CodeTerm>);
pub type Query = Vec<CodeTerm>;
pub type Program = Vec<Clause>;

pub fn solve(program: Program, query: Query) {
    let (vars, heap_query, var_map) = VarArena::new(query);

    let mut context = Context {
        goals: heap_query.clone().into(),
        vars,
        trail: Trail::new(),
    };

    context.solve(&program, &var_map)
}

pub struct Context {
    goals: VecDeque<HeapTermPtr>,
    vars: VarArena,
    trail: Trail,
}

impl Context {
    fn solve(&mut self, program: &Program, var_map: &Vec<(VarName, HeapTermPtr)>) {
        let goal = self.goals.pop_front().unwrap();

        for clause in program {
            let trail_checkpoint = self.trail.checkpoint();
            let arena_checkpoint = self.vars.checkpoint();

            let (head, body) = self.vars.alloc_clause(clause);

            if self.unify(goal, head) {
                body.iter()
                    .rev()
                    .for_each(|goal| self.goals.push_front(*goal));

                if self.goals.is_empty() {
                    print!("yay: ");
                    for (name, var) in var_map {
                        print!("{}={:?} ", name, self.vars.serialize(*var));
                    }
                    println!();
                } else {
                    self.solve(program, var_map);
                }
            } else {
                println!("no");
            }

            self.trail.undo(trail_checkpoint, &mut self.vars);
            self.vars.undo(arena_checkpoint)
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
            (HeapTerm::Compound(f, a_arity), HeapTerm::Compound(g, b_arity)) => {
                if f != g || a_arity != b_arity {
                    return false;
                }

                let checkpoint = self.trail.checkpoint();

                for (a, b) in (a_ptr + 1..a_ptr + 1 + a_arity).zip(b_ptr + 1..b_ptr + 1 + b_arity) {
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
}
