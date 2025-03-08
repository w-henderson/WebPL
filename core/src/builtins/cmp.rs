use crate::builtins::{arithmetic, Builtin, BuiltinError};
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

pub struct EquivBuiltin;

impl Builtin<2> for EquivBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        Ok(equiv(solver, args, args + 1))
    }
}

macro_rules! impl_arithmetic_cmp {
    ($op:ident, $method:ident) => {
        pub struct $op;

        impl Builtin<2> for $op {
            fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
                let a = arithmetic::eval(solver, args)?;
                let b = arithmetic::eval(solver, args + 1)?;

                match (a, b) {
                    (Atom::Integer(a), Atom::Integer(b)) => Ok(a.$method(&b)),
                    (Atom::Float(a), Atom::Float(b)) => Ok(a.$method(&b)),
                    (Atom::Integer(a), Atom::Float(b)) => Ok((a as f64).$method(&b)),
                    (Atom::Float(a), Atom::Integer(b)) => Ok(a.$method(&(b as f64))),
                    (Atom::Integer(_), _) | (Atom::Float(_), _) => {
                        Err(BuiltinError::NotANumber(args + 1))
                    }
                    _ => Err(BuiltinError::NotANumber(args)),
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
    let a_root = solver.heap.get_ptr(a);
    let b_root = solver.heap.get_ptr(b);

    match (solver.heap.get(a_root), solver.heap.get(b_root)) {
        (HeapTerm::Atom(a), HeapTerm::Atom(b)) => a == b,
        (HeapTerm::Var(a, _, _, _), HeapTerm::Var(b, _, _, _)) => a == b,
        (HeapTerm::Compound(f, a), HeapTerm::Compound(g, b)) => {
            f == g && a == b && (1..=*a).all(|i| equiv(solver, a_root + i, b_root + i))
        }
        (HeapTerm::Cut(_), HeapTerm::Cut(_)) => true,
        (HeapTerm::Lambda(js1, a1), HeapTerm::Lambda(js2, a2)) => {
            js1 == js2 && a1 == a2 && (1..=*a1).all(|i| equiv(solver, a_root + i, b_root + i))
        }
        _ => false,
    }
}
