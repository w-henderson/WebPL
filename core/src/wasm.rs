use wasm_bindgen::prelude::*;

use crate::Error;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
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
    pub fn next(&mut self) -> Result<Option<js_sys::Map>, Error> {
        self.0.step().map(|o| o.map(solution_to_js))
    }

    #[wasm_bindgen]
    pub fn all(&mut self) -> Result<js_sys::Array, Error> {
        self.0.by_ref().map(|s| s.map(solution_to_js)).collect()
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
