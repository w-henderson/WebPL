use crate::builtins::BuiltinError;
use crate::{Atom, HeapTerm, HeapTermPtr, LambdaId, Solver};

use crate::wasm::{eval_js, Term};

use wasm_bindgen::JsValue;

use std::cell::RefCell;

pub fn eval(
    solver: &mut Solver,
    id: LambdaId,
    args: Vec<HeapTermPtr>,
) -> Result<bool, BuiltinError> {
    let lambda = solver.lambdas[id].clone();
    let arg_values: Vec<JsValue> = args
        .into_iter()
        .map(|ptr| JsValue::from(Term::from_heap(&solver.heap, ptr)))
        .collect();

    let solver = RefCell::new(solver);

    let result = eval_js(
        lambda.js,
        lambda.arg_names,
        arg_values,
        &mut |a, b| solver.borrow_mut().unify(a, b),
        &mut |a| {
            if let Some(a) = a.as_f64() {
                if a == a as i64 as f64 {
                    Ok(solver
                        .borrow_mut()
                        .heap
                        .alloc(HeapTerm::Atom(Atom::Integer(a as i64))))
                } else {
                    Ok(solver
                        .borrow_mut()
                        .heap
                        .alloc(HeapTerm::Atom(Atom::Float(a))))
                }
            } else if let Some(a) = a.as_string() {
                let a = solver.borrow_mut().heap.string_map.alloc(&a);
                Ok(solver
                    .borrow_mut()
                    .heap
                    .alloc(HeapTerm::Atom(Atom::String(a))))
            } else {
                Err("Can only allocate numbers and strings".to_string())
            }
        },
    );

    result.map_err(|e| {
        BuiltinError::JavaScriptError(e.as_string().unwrap_or_else(|| "<js error>".to_string()))
    })
}
