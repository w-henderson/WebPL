use crate::builtins::{arithmetic, Builtin, BuiltinError};
use crate::{HeapTermPtr, Solver};

pub struct IsBuiltin;

impl Builtin<2> for IsBuiltin {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
        let result = arithmetic::eval(solver, args[1]);

        result.map(|n| {
            let n: HeapTermPtr = solver.heap.alloc_atom(n);
            solver.unify(args[0], n)
        })
    }
}
