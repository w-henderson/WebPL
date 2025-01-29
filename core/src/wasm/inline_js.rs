use crate::builtins::BuiltinError;
use crate::{Atom, HeapTermPtr, Solver};

use crate::wasm::{eval_js, Term};

use wasm_bindgen::JsValue;

use std::cell::RefCell;

pub fn eval(solver: &mut Solver, js: usize, args: Vec<HeapTermPtr>) -> Result<bool, BuiltinError> {
    let js = solver.heap.get_atom(js).to_string();
    let args: Vec<JsValue> = args
        .into_iter()
        .map(|ptr| JsValue::from(Term::from_heap(&solver.heap, ptr)))
        .collect();

    let solver = RefCell::new(solver);

    let result = eval_js(
        &js,
        args,
        &mut |a, b| solver.borrow_mut().unify(a, b),
        &mut |a| {
            if let Some(a) = a.as_f64() {
                if a == a as i64 as f64 {
                    solver.borrow_mut().heap.alloc_atom(Atom::Integer(a as i64))
                } else {
                    solver.borrow_mut().heap.alloc_atom(Atom::Float(a))
                }
            } else if let Some(a) = a.as_string() {
                let a = solver.borrow_mut().heap.string_map.alloc(&a);
                solver.borrow_mut().heap.alloc_atom(Atom::String(a))
            } else {
                panic!("Invalid term")
            }
        },
    );

    Ok(result)
}
