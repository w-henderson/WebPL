use crate::builtins::{arithmetic, Builtin, BuiltinError};
use crate::{Atom, HeapTermPtr, Solver};

macro_rules! impl_arithmetic_cmp {
    ($op:ident, $method:ident) => {
        pub struct $op;

        impl Builtin<2> for $op {
            fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
                let a = arithmetic::eval(solver, args[0])?;
                let b = arithmetic::eval(solver, args[1])?;

                match (a, b) {
                    (Atom::Integer(a), Atom::Integer(b)) => Ok(a.$method(&b)),
                    (Atom::Float(a), Atom::Float(b)) => Ok(a.$method(&b)),
                    (Atom::Integer(a), Atom::Float(b)) => Ok((a as f64).$method(&b)),
                    (Atom::Float(a), Atom::Integer(b)) => Ok(a.$method(&(b as f64))),
                    (Atom::Integer(_), _) | (Atom::Float(_), _) => {
                        Err(BuiltinError::NotANumber(args[1]))
                    }
                    _ => Err(BuiltinError::NotANumber(args[0])),
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
