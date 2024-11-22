mod cmp;
mod is;
mod unify;

use crate::{HeapTerm, HeapTermPtr, Solver};

#[derive(Debug)]
pub enum BuiltinError {
    NotANumber,
    InsufficientlyInstantiated,
    UnsupportedOperation,
}

pub trait Builtin<const ARITY: usize> {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; ARITY]) -> Result<bool, BuiltinError>;
}

pub fn eval(solver: &mut Solver, goal: HeapTermPtr) -> Option<Result<bool, BuiltinError>> {
    if let HeapTerm::Compound(functor, arity, next) = solver.vars.get(goal) {
        match (solver.vars.get_atom(*functor), arity) {
            ("=", 2) => Some(unify::UnifyBuiltin::eval(solver, args(solver, *next))),
            ("is", 2) => Some(is::IsBuiltin::eval(solver, args(solver, *next))),
            (">", 2) => Some(cmp::GtBuiltin::eval(solver, args(solver, *next))),
            (">=", 2) => Some(cmp::GteBuiltin::eval(solver, args(solver, *next))),
            ("<", 2) => Some(cmp::LtBuiltin::eval(solver, args(solver, *next))),
            ("<=", 2) => Some(cmp::LteBuiltin::eval(solver, args(solver, *next))),
            ("=\\=", 2) => Some(cmp::NeqBuiltin::eval(solver, args(solver, *next))),
            ("=:=", 2) => Some(cmp::EqBuiltin::eval(solver, args(solver, *next))),
            _ => None,
        }
    } else {
        None
    }
}

pub fn args<const N: usize>(solver: &Solver, next: Option<HeapTermPtr>) -> [HeapTermPtr; N] {
    let mut args = [0; N];
    let mut i = 0;
    let mut next = next;

    while let Some(arg) = next {
        match solver.vars.get(arg) {
            HeapTerm::CompoundCons(head, tail) => {
                args[i] = *head;
                i += 1;
                next = *tail;
            }
            _ => break,
        }
    }

    debug_assert_eq!(i, N);

    args
}
