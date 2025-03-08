use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTermPtr, Solver};

pub struct CallBuiltin;

impl Builtin<1> for CallBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        solver.goals.push_pending(solver.heap.get_ptr(args));
        Ok(true)
    }
}
