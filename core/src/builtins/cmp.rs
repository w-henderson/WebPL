use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTerm, HeapTermPtr, Solver};

macro_rules! impl_arithmetic_cmp {
    ($op:ident, $method:ident) => {
        pub struct $op;

        impl Builtin<2> for $op {
            fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
                let a = solver.vars.get(args[0]);
                let b = solver.vars.get(args[1]);

                match (a, b) {
                    (HeapTerm::Atom(a), HeapTerm::Atom(b)) => {
                        let a = solver.vars.get_atom(*a);
                        let b = solver.vars.get_atom(*b);
                        Ok(a.parse::<f64>()
                            .map_err(|_| BuiltinError::NotANumber)?
                            .$method(&b.parse::<f64>().map_err(|_| BuiltinError::NotANumber)?))
                    }
                    _ => Err(BuiltinError::InsufficientlyInstantiated),
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
