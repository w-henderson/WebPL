use crate::builtins::{Builtin, BuiltinError};
use crate::{Atom, HeapTerm, HeapTermPtr, Solver};

macro_rules! impl_type_check {
    ($name:ident, $matcher:pat) => {
        pub struct $name;

        impl Builtin<1> for $name {
            fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
                Ok(matches!(solver.heap.get(args), $matcher))
            }
        }
    };
}

impl_type_check!(IsIntegerBuiltin, HeapTerm::Atom(Atom::Integer(_)));
impl_type_check!(IsFloatBuiltin, HeapTerm::Atom(Atom::Float(_)));
impl_type_check!(IsAtomBuiltin, HeapTerm::Atom(_));
impl_type_check!(
    IsNumberBuiltin,
    HeapTerm::Atom(Atom::Integer(_)) | HeapTerm::Atom(Atom::Float(_))
);
impl_type_check!(IsVarBuiltin, HeapTerm::Var(_, _, _));
impl_type_check!(IsCompoundBuiltin, HeapTerm::Compound(_, _));
