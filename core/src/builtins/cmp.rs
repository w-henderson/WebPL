use crate::builtins::{arithmetic, Builtin, BuiltinError};
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

pub struct EquivBuiltin;

impl Builtin<2> for EquivBuiltin {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
        Ok(equiv(solver, args[0], args[1]))
    }
}

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

fn equiv(solver: &Solver, a: HeapTermPtr, b: HeapTermPtr) -> bool {
    match (solver.heap.get(a), solver.heap.get(b)) {
        (HeapTerm::Atom(a), HeapTerm::Atom(b)) => a == b,
        (HeapTerm::Var(a, _), HeapTerm::Var(b, _)) => a == b,
        (HeapTerm::Compound(f, a, n1), HeapTerm::Compound(g, b, n2)) => {
            f == g
                && a == b
                && match (n1, n2) {
                    (Some(n1), Some(n2)) => equiv(solver, *n1, *n2),
                    _ => false,
                }
        }
        (HeapTerm::CompoundCons(h1, t1), HeapTerm::CompoundCons(h2, t2)) => {
            equiv(solver, *h1, *h2)
                && match (t1, t2) {
                    (Some(t1), Some(t2)) => equiv(solver, *t1, *t2),
                    _ => false,
                }
        }
        (HeapTerm::Cut(_), HeapTerm::Cut(_)) => true,
        (HeapTerm::Lambda(js1, a1, n1), HeapTerm::Lambda(js2, a2, n2)) => {
            js1 == js2
                && a1 == a2
                && match (n1, n2) {
                    (Some(n1), Some(n2)) => equiv(solver, *n1, *n2),
                    _ => false,
                }
        }
        _ => false,
    }
}
