use crate::builtins::{Builtin, BuiltinError};
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

macro_rules! impl_arithmetic_cmp {
    ($op:ident, $method:ident) => {
        pub struct $op;

        impl Builtin<2> for $op {
            fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
                let a = solver.heap.get(args[0]);
                let b = solver.heap.get(args[1]);

                match (a, b) {
                    (HeapTerm::Atom(Atom::Integer(a)), HeapTerm::Atom(Atom::Integer(b))) => {
                        Ok(a.$method(b))
                    }
                    (HeapTerm::Atom(Atom::Float(a)), HeapTerm::Atom(Atom::Float(b))) => {
                        Ok(a.$method(b))
                    }
                    (HeapTerm::Atom(Atom::Integer(a)), HeapTerm::Atom(Atom::Float(b))) => {
                        Ok((*a as f64).$method(b))
                    }
                    (HeapTerm::Atom(Atom::Float(a)), HeapTerm::Atom(Atom::Integer(b))) => {
                        Ok(a.$method(&(*b as f64)))
                    }
                    (HeapTerm::Var(_), _) => Err(BuiltinError::InsufficientlyInstantiated(args[0])),
                    (_, HeapTerm::Var(_)) => Err(BuiltinError::InsufficientlyInstantiated(args[1])),
                    (HeapTerm::Atom(Atom::Integer(_)), _) | (HeapTerm::Atom(Atom::Float(_)), _) => {
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
