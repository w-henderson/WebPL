mod is;
mod unify;

use crate::{HeapTerm, HeapTermPtr, Solver};

pub trait Builtin<const ARITY: usize> {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; ARITY]) -> Result<bool, ()>;
}

pub fn eval(solver: &mut Solver, goal: HeapTermPtr) -> Result<bool, ()> {
    if let HeapTerm::Compound(functor, args) = solver.vars.get(goal) {
        match (functor.as_str(), args.len()) {
            ("=", 2) => unify::UnifyBuiltin::eval(solver, [args[0], args[1]]),
            ("is", 2) => is::IsBuiltin::eval(solver, [args[0], args[1]]),
            _ => Ok(false),
        }
    } else {
        Ok(false)
    }
}
