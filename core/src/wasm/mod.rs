#[cfg(target_family = "wasm")]
pub mod inline_js;

#[cfg(not(target_family = "wasm"))]
pub mod inline_js {
    pub fn eval(
        _: &mut crate::Solver,
        _: usize,
        _: Vec<crate::HeapTermPtr>,
    ) -> Result<bool, crate::builtins::BuiltinError> {
        Err(crate::builtins::BuiltinError::UnsupportedPlatform)
    }
}

use serde::{ser::SerializeStruct, Serialize};
use wasm_bindgen::prelude::*;

use crate::{Atom, Error, Heap, HeapTerm, HeapTermPtr};

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(module = "/src/wasm/inline_js_handler.js")]
extern "C" {
    fn eval_js(
        js: &str,
        args: Vec<JsValue>,
        unify_wasm: &mut dyn FnMut(HeapTermPtr, HeapTermPtr) -> bool,
        alloc_wasm: &mut dyn FnMut(JsValue) -> HeapTermPtr,
    ) -> bool;
}

pub enum Term {
    String(String),
    Number(f64),
    Variable(HeapTermPtr),
    Compound(String, Vec<Term>),
}

#[wasm_bindgen]
pub struct Solver(crate::Solver);

#[wasm_bindgen]
impl Solver {
    #[wasm_bindgen(constructor)]
    pub fn new(program: &str, query: &str) -> Result<Solver, JsValue> {
        Ok(Solver(crate::Solver::new(program, query)?))
    }

    #[wasm_bindgen]
    pub fn new_with_gc(program: &str, query: &str) -> Result<Solver, JsValue> {
        Ok(Solver(crate::Solver::new_with_gc(program, query)?))
    }

    #[wasm_bindgen]
    pub fn next(&mut self) -> Result<Option<js_sys::Map>, Error> {
        self.0.step().map(|o| o.map(solution_to_js))
    }

    #[wasm_bindgen]
    pub fn all(&mut self) -> Result<js_sys::Array, Error> {
        self.0.by_ref().map(|s| s.map(solution_to_js)).collect()
    }
}

impl Term {
    pub fn from_heap(heap: &Heap, ptr: HeapTermPtr) -> Self {
        match heap.get(ptr) {
            HeapTerm::Atom(Atom::String(id)) => Term::String(heap.get_atom(*id).to_string()),
            HeapTerm::Atom(Atom::Integer(i)) => Term::Number(*i as f64),
            HeapTerm::Atom(Atom::Float(f)) => Term::Number(*f),
            HeapTerm::Var(ptr, _) => Term::Variable(*ptr),
            HeapTerm::Compound(functor, arity, next) => Term::Compound(
                heap.get_atom(*functor).to_string(),
                crate::builtins::dyn_args(heap, *arity, *next)
                    .into_iter()
                    .map(|ptr| Term::from_heap(heap, ptr))
                    .collect::<Vec<_>>(),
            ),
            _ => panic!("Invalid term"),
        }
    }
}

impl Serialize for Term {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Term::String(s) => s.serialize(serializer),
            Term::Number(n) => n.serialize(serializer),
            Term::Variable(ptr) => {
                let mut obj = serializer.serialize_struct("variable", 1)?;
                obj.serialize_field("variable", ptr)?;
                obj.end()
            }
            Term::Compound(functor, args) => {
                let mut obj = serializer.serialize_struct("compound", 2)?;
                obj.serialize_field("functor", functor)?;
                obj.serialize_field("args", args)?;
                obj.end()
            }
        }
    }
}

impl From<Term> for JsValue {
    fn from(value: Term) -> Self {
        serde_wasm_bindgen::to_value(&value).unwrap()
    }
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        serde_wasm_bindgen::to_value(&value).unwrap()
    }
}

fn solution_to_js(solution: crate::Solution) -> js_sys::Map {
    let result = js_sys::Map::new();
    for (k, v) in solution {
        result.set(&k.into(), &v.into());
    }
    result
}
