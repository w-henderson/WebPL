use crate::builtins::BuiltinError;
use crate::stringmap::str;
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

use std::ops::{Add, Div, Mul, Rem, Shl, Shr, Sub};

pub fn eval(solver: &mut Solver, term: HeapTermPtr) -> Result<Atom, BuiltinError> {
    let term_ptr = solver.heap.get_ptr(term);
    match solver.heap.get(term_ptr) {
        HeapTerm::Atom(atom) => {
            if matches!(atom, Atom::Integer(_) | Atom::Float(_)) {
                Ok(*atom)
            } else {
                Err(BuiltinError::NotANumber(term))
            }
        }
        HeapTerm::Var(_, _, _, _) => Err(BuiltinError::InsufficientlyInstantiated(term)),
        HeapTerm::Compound(f, arity) if *arity == 2 => {
            let f = *f;
            let a = eval(solver, term_ptr + 1)?;
            let b = eval(solver, term_ptr + 2)?;

            match f {
                str::ADD => add(&a, &b),
                str::SUB => sub(&a, &b),
                str::MUL => mul(&a, &b),
                str::DIV => div(&a, &b),
                str::INTDIV => div_euclid(&a, &b),
                str::MOD => rem(&a, &b),
                str::RSHIFT => shr(&a, &b),
                str::LSHIFT => shl(&a, &b),
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
    (NO FLOAT $op:ident) => {
        fn $op(a: &Atom, b: &Atom) -> Result<Atom, ()> {
            match (a, b) {
                (Atom::Integer(a), Atom::Integer(b)) => Ok(Atom::Integer(a.$op(*b))),
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
impl_arithmetic_op!(NO FLOAT shl);
impl_arithmetic_op!(NO FLOAT shr);
