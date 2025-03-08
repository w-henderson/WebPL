use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTerm, HeapTermPtr, Solver};

pub struct DelayBuiltin;

impl Builtin<2> for DelayBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        let var = solver.heap.get_ptr(args);
        let goal = solver.heap.get_ptr(args + 1);

        match &mut solver.heap.data[var] {
            HeapTerm::Var(_, _, attribute) => *attribute = goal,
            _ => solver.goals.push_pending(goal),
        }

        Ok(true)
    }
}
