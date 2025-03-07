mod arithmetic;
mod cmp;
mod is;
mod statistics;
mod types;
mod unify;

use crate::stringmap::str;
use crate::{Error, HeapTerm, HeapTermPtr, Solver, StringId};

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinError {
    NotANumber(HeapTermPtr),
    InsufficientlyInstantiated(HeapTermPtr),
    UnsupportedOperation(StringId),
    UnsupportedPlatform,
    JavaScriptError(String),
}

pub trait Builtin<const ARITY: usize> {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError>;
}

pub fn eval(solver: &mut Solver, goal: HeapTermPtr) -> Option<Result<bool, BuiltinError>> {
    let goal_ptr = solver.heap.get_ptr(goal);
    match solver.heap.get(goal_ptr) {
        HeapTerm::Compound(functor, arity) => {
            if *arity == 1 {
                match *functor {
                    str::INTEGER => Some(types::IsIntegerBuiltin::eval(solver, goal_ptr + 1)),
                    str::FLOAT => Some(types::IsFloatBuiltin::eval(solver, goal_ptr + 1)),
                    str::ATOM => Some(types::IsAtomBuiltin::eval(solver, goal_ptr + 1)),
                    str::COMPOUND => Some(types::IsCompoundBuiltin::eval(solver, goal_ptr + 1)),
                    str::NUMBER => Some(types::IsNumberBuiltin::eval(solver, goal_ptr + 1)),
                    str::VAR => Some(types::IsVarBuiltin::eval(solver, goal_ptr + 1)),
                    _ => None,
                }
            } else if *arity == 2 {
                match *functor {
                    str::EQ => Some(unify::UnifyBuiltin::eval(solver, goal_ptr + 1)),
                    str::IS => Some(is::IsBuiltin::eval(solver, goal_ptr + 1)),
                    str::GT => Some(cmp::GtBuiltin::eval(solver, goal_ptr + 1)),
                    str::GE => Some(cmp::GteBuiltin::eval(solver, goal_ptr + 1)),
                    str::LT => Some(cmp::LtBuiltin::eval(solver, goal_ptr + 1)),
                    str::LE => Some(cmp::LteBuiltin::eval(solver, goal_ptr + 1)),
                    str::ANE => Some(cmp::NeqBuiltin::eval(solver, goal_ptr + 1)),
                    str::AEQ => Some(cmp::EqBuiltin::eval(solver, goal_ptr + 1)),
                    str::STAT => Some(statistics::StatisticsBuiltin::eval(solver, goal_ptr + 1)),
                    str::EQUIV => Some(cmp::EquivBuiltin::eval(solver, goal_ptr + 1)),
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
        HeapTerm::Lambda(id, arity) => {
            let args = (1..=*arity).map(|i| goal_ptr + i).collect();
            Some(crate::wasm::inline_js::eval(solver, *id, args))
        }
        _ => None,
    }
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
            BuiltinError::UnsupportedPlatform => "Unsupported platform, requires WASM".to_string(),
            BuiltinError::JavaScriptError(e) => format!("JS: {}", e),
        },
    }
}
