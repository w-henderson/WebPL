use crate::builtins::Builtin;
use crate::{HeapTerm, HeapTermPtr, Solver};

macro_rules! impl_arithmetic_cmp {
    ($op:ident, $method:ident) => {
        pub struct $op;

        impl Builtin<2> for $op {
            fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, ()> {
                let a = solver.vars.get(args[0]);
                let b = solver.vars.get(args[1]);

                match (a, b) {
                    (HeapTerm::Atom(a), HeapTerm::Atom(b)) => Ok(a
                        .parse::<f64>()
                        .map_err(|_| ())?
                        .$method(&b.parse::<f64>().map_err(|_| ())?)),
                    _ => Err(()), // insufficiently instantiated or unsupported operation
                }
            }
        }
    };
}

impl_arithmetic_cmp!(EqBuiltin, eq);
impl_arithmetic_cmp!(NeqBuiltin, ne);
impl_arithmetic_cmp!(GtBuiltin, gt);
impl_arithmetic_cmp!(GteBuiltin, ge);
impl_arithmetic_cmp!(LtBuiltin, lt);
impl_arithmetic_cmp!(LteBuiltin, le);
