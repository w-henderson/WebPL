use crate::atom::Atom;
use crate::builtins::{Builtin, BuiltinError};
use crate::{HeapTerm, HeapTermPtr, Solver};

pub struct StatisticsBuiltin;

impl Builtin<2> for StatisticsBuiltin {
    fn eval(solver: &mut Solver, args: HeapTermPtr) -> Result<bool, BuiltinError> {
        Ok(match solver.heap.get(args) {
            HeapTerm::Atom(Atom::String(id)) => match solver.heap.get_atom(*id) {
                "memory" => unify_int(solver, args + 1, solver.heap.size() as i64),
                "allocated" => unify_int(solver, args + 1, solver.heap.capacity() as i64),
                _ => false,
            },
            _ => false,
        })
    }
}

fn unify_int(solver: &mut Solver, a: HeapTermPtr, i: i64) -> bool {
    let atom: HeapTermPtr = solver.heap.alloc_atom(Atom::Integer(i));
    solver.unify(a, atom)
}
