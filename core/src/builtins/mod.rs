mod cmp;
mod is;
mod statistics;
mod unify;

use crate::stringmap::str;
use crate::{Error, HeapTerm, HeapTermPtr, Solver, StringId};

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinError {
    NotANumber(HeapTermPtr),
    InsufficientlyInstantiated(HeapTermPtr),
    UnsupportedOperation(StringId),
}

pub trait Builtin<const ARITY: usize> {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; ARITY]) -> Result<bool, BuiltinError>;
}

pub fn eval(solver: &mut Solver, goal: HeapTermPtr) -> Option<Result<bool, BuiltinError>> {
    match solver.heap.get(goal) {
        HeapTerm::Compound(functor, arity, next) => {
            if *arity == 2 {
                let args = args(solver, *next);
                match *functor {
                    str::EQ => Some(unify::UnifyBuiltin::eval(solver, args)),
                    str::IS => Some(is::IsBuiltin::eval(solver, args)),
                    str::GT => Some(cmp::GtBuiltin::eval(solver, args)),
                    str::GE => Some(cmp::GteBuiltin::eval(solver, args)),
                    str::LT => Some(cmp::LtBuiltin::eval(solver, args)),
                    str::LE => Some(cmp::LteBuiltin::eval(solver, args)),
                    str::ANE => Some(cmp::NeqBuiltin::eval(solver, args)),
                    str::AEQ => Some(cmp::EqBuiltin::eval(solver, args)),
                    str::STAT => Some(statistics::StatisticsBuiltin::eval(solver, args)),
                    _ => None,
                }
            } else {
                None
            }
        }
        HeapTerm::Cut(choice_point_idx) => {
            solver.cut(*choice_point_idx);
            Some(Ok(true))
        }
        _ => None,
    }
}

pub fn args<const N: usize>(solver: &Solver, next: Option<HeapTermPtr>) -> [HeapTermPtr; N] {
    let mut args = [0; N];
    let mut i = 0;
    let mut next = next;

    while let Some(arg) = next {
        match solver.heap.get(arg) {
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

pub fn error(solver: &Solver, error: BuiltinError) -> Error {
    Error {
        location: None,
        error: match error {
            BuiltinError::NotANumber(ptr) => {
                format!(
                    "Expected a number, got `{}`",
                    solver.heap.serialize(&[("Err".to_string(), ptr)])[0].1
                )
            }
            BuiltinError::InsufficientlyInstantiated(ptr) => format!(
                "Insufficiently instantiated variable `{}`",
                solver.heap.serialize(&[("Err".to_string(), ptr)])[0].1
            ),
            BuiltinError::UnsupportedOperation(s) => {
                format!("Unsupported operation `{}`", solver.heap.get_atom(s))
            }
        },
    }
}
