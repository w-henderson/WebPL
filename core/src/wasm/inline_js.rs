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
        &mut |a, b| unify_wasm(&mut solver.borrow_mut(), a, b),
        &mut |a| alloc_wasm(&mut solver.borrow_mut(), a),
    );

    result.map_err(|e| {
        BuiltinError::JavaScriptError(e.as_string().unwrap_or_else(|| "<js error>".to_string()))
    })
}

fn unify_wasm(solver: &mut Solver, a: HeapTermPtr, b: HeapTermPtr) -> bool {
    solver.unify(a, b)
}

fn alloc_wasm(solver: &mut Solver, a: JsValue) -> Result<HeapTermPtr, String> {
    if let Some(a) = a.as_f64() {
        if a == a as i64 as f64 {
            Ok(solver.heap.alloc(HeapTerm::Atom(Atom::Integer(a as i64))))
        } else {
            Ok(solver.heap.alloc(HeapTerm::Atom(Atom::Float(a))))
        }
    } else if let Some(a) = a.as_string() {
        let a = solver.heap.string_map.alloc(&a);
        Ok(solver.heap.alloc(HeapTerm::Atom(Atom::String(a))))
    } else if let Some(a) = js_sys::Reflect::get(&a, &JsValue::from_str("variable"))
        .ok()
        .and_then(|a| a.as_f64())
    {
        if (a as usize) < solver.heap.data.len() {
            Ok(a as usize)
        } else {
            Err("Invalid variable".to_string())
        }
    } else if let Some(functor) = js_sys::Reflect::get(&a, &JsValue::from_str("functor"))
        .ok()
        .and_then(|a| a.as_string())
    {
        let functor = solver.heap.string_map.alloc(&functor);
        let args = js_sys::Reflect::get(&a, &JsValue::from_str("args"))
            .ok()
            .ok_or("No args")?;
        let args = js_sys::Array::from(&args);

        let ptr = solver
            .heap
            .alloc(HeapTerm::Compound(functor, args.length() as usize));

        let args_heap = solver.heap.data.len();
        for _ in 0..args.length() {
            solver.heap.alloc_new_var();
        }

        for (i, arg) in args.iter().enumerate() {
            let arg = alloc_wasm(solver, arg)?;
            solver.heap.data[args_heap + i] = HeapTerm::Var(arg, false, false, 0);
        }

        Ok(ptr)
    } else {
        Err("Invalid type".to_string())
    }
}
