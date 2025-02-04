use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTermPtr, Solver};

pub struct UnifyBuiltin;

impl Builtin<2> for UnifyBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        Ok(solver.unify(args, args + 1))
    }
}
