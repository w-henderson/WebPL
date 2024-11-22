use crate::builtins::{args, Builtin, BuiltinError};
use crate::{HeapTerm, HeapTermPtr, Solver};

pub struct IsBuiltin;

pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Builtin<2> for IsBuiltin {
    fn eval(solver: &mut Solver, args: [HeapTermPtr; 2]) -> Result<bool, BuiltinError> {
        let result = IsBuiltin::arithmetic_eval(solver, args[1]);

        result.map(|n| {
            let n: HeapTermPtr = solver.vars.alloc_atom(n.to_string());
            solver.unify(args[0], n)
        })
    }
}

impl IsBuiltin {
    fn arithmetic_eval(solver: &mut Solver, term: HeapTermPtr) -> Result<Number, BuiltinError> {
        match solver.vars.get(term) {
            HeapTerm::Atom(atom) => Self::arithmetic_eval_atom(atom),
            HeapTerm::Var(_) => Err(BuiltinError::InsufficientlyInstantiated),
            HeapTerm::Compound(f, arity, next) if *arity == 2 => {
                let f = f.clone();
                let args = args::<2>(solver, *next);
                let a = Self::arithmetic_eval(solver, args[0])?;
                let b = Self::arithmetic_eval(solver, args[1])?;

                match f.as_str() {
                    "+" => Ok(a + b),
                    "-" => Ok(a - b),
                    "*" => Ok(a * b),
                    "/" => Ok(a / b),
                    _ => Err(BuiltinError::UnsupportedOperation),
                }
            }
            _ => Err(BuiltinError::UnsupportedOperation),
        }
    }

    fn arithmetic_eval_atom(atom: &str) -> Result<Number, BuiltinError> {
        if let Ok(int) = atom.parse::<i64>() {
            Ok(Number::Integer(int))
        } else if let Ok(float) = atom.parse::<f64>() {
            Ok(Number::Float(float))
        } else {
            Err(BuiltinError::NotANumber)
        }
    }
}

macro_rules! impl_arithmetic_op {
    ($op:ident, $method:ident) => {
        impl std::ops::$op for Number {
            type Output = Number;

            fn $method(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Number::Integer(a), Number::Integer(b)) => Number::Integer(a.$method(b)),
                    (Number::Float(a), Number::Float(b)) => Number::Float(a.$method(b)),
                    (Number::Integer(a), Number::Float(b)) => Number::Float((a as f64).$method(b)),
                    (Number::Float(a), Number::Integer(b)) => Number::Float(a.$method(b as f64)),
                }
            }
        }
    };
}

impl_arithmetic_op!(Add, add);
impl_arithmetic_op!(Sub, sub);
impl_arithmetic_op!(Mul, mul);
impl_arithmetic_op!(Div, div);

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Number::Integer(n) => write!(f, "{}", n),
            Number::Float(n) => write!(f, "{}", n),
        }
    }
}
