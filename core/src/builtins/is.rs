use crate::builtins::{arithmetic, Builtin, BuiltinError};
use crate::{HeapTerm, HeapTermPtr, Solver};

pub struct IsBuiltin;

impl Builtin<2> for IsBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        let result = arithmetic::eval(solver, args + 1);

        result.map(|n| {
            let n: HeapTermPtr = solver.heap.alloc(HeapTerm::Atom(n));
            solver.unify(args, n)
        })
    }
}
