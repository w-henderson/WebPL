use crate::builtins::Builtin;
use crate::{HeapTermPtr, Solver};

pub struct UnifyBuiltin;

impl Builtin<2> for UnifyBuiltin {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, ()> {
        assert!(solver.unify(args[0], args[1]));
        Ok(true)
    }
}
