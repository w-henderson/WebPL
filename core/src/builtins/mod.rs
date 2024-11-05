mod cmp;
mod is;
mod unify;

use crate::{HeapTerm, HeapTermPtr, Solver};

pub trait Builtin<const ARITY: usize> {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; ARITY]) -> Result<bool, ()>;
}

pub fn eval(solver: &mut Solver, goal: HeapTermPtr) -> Option<Result<bool, ()>> {
    if let HeapTerm::Compound(functor, args) = solver.vars.get(goal) {
        match (functor.as_str(), args.len()) {
            ("=", 2) => Some(unify::UnifyBuiltin::eval(solver, [args[0], args[1]])),
            ("is", 2) => Some(is::IsBuiltin::eval(solver, [args[0], args[1]])),
            (">", 2) => Some(cmp::GtBuiltin::eval(solver, [args[0], args[1]])),
            (">=", 2) => Some(cmp::GteBuiltin::eval(solver, [args[0], args[1]])),
            ("<", 2) => Some(cmp::LtBuiltin::eval(solver, [args[0], args[1]])),
            ("<=", 2) => Some(cmp::LteBuiltin::eval(solver, [args[0], args[1]])),
            ("=\\=", 2) => Some(cmp::NeqBuiltin::eval(solver, [args[0], args[1]])),
            ("=:=", 2) => Some(cmp::EqBuiltin::eval(solver, [args[0], args[1]])),
            _ => None,
        }
    } else {
        None
    }
}
