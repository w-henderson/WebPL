use crate::builtins::{args, BuiltinError};
use crate::stringmap::str;
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

use std::ops::{Add, Div, Mul, Rem, Sub};

pub fn eval(solver: &mut Solver, term: HeapTermPtr) -> Result<Atom, BuiltinError> {
    match solver.heap.get(term) {
        HeapTerm::Atom(atom) => {
            if matches!(atom, Atom::Integer(_) | Atom::Float(_)) {
                Ok(*atom)
            } else {
                Err(BuiltinError::NotANumber(term))
            }
        }
        HeapTerm::Var(_, _) => Err(BuiltinError::InsufficientlyInstantiated(term)),
        HeapTerm::Compound(f, arity, next) if *arity == 2 => {
            let f = *f;
            let args = args::<2>(solver, *next);
            let a = eval(solver, args[0])?;
            let b = eval(solver, args[1])?;

            match f {
                str::ADD => add(&a, &b),
                str::SUB => sub(&a, &b),
                str::MUL => mul(&a, &b),
                str::DIV => div(&a, &b),
                str::INTDIV => div_euclid(&a, &b),
                str::MOD => rem(&a, &b),
                _ => Err(()),
            }
            .map_err(|_| BuiltinError::UnsupportedOperation(f))
        }
        _ => Err(BuiltinError::NotANumber(term)),
    }
}

macro_rules! impl_arithmetic_op {
    ($op:ident) => {
        fn $op(a: &Atom, b: &Atom) -> Result<Atom, ()> {
            match (a, b) {
                (Atom::Integer(a), Atom::Integer(b)) => Ok(Atom::Integer(a.$op(*b))),
                (Atom::Float(a), Atom::Float(b)) => Ok(Atom::Float(a.$op(*b))),
                (Atom::Integer(a), Atom::Float(b)) => Ok(Atom::Float((*a as f64).$op(*b))),
                (Atom::Float(a), Atom::Integer(b)) => Ok(Atom::Float(a.$op(*b as f64))),
                _ => Err(()),
            }
        }
    };
}

impl_arithmetic_op!(add);
impl_arithmetic_op!(sub);
impl_arithmetic_op!(mul);
impl_arithmetic_op!(div);
impl_arithmetic_op!(div_euclid);
impl_arithmetic_op!(rem);
