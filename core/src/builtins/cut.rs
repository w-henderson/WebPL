use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTermPtr, Solver};

pub struct CutBuiltin;

impl Builtin<0> for CutBuiltin {
    fn eval(solver: &mut Solver, _args: [HeapTermPtr; 0]) -> Result<bool, BuiltinError> {
        solver.choice_points.clear();
        Ok(true)
    }
}
