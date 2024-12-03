use wasm_bindgen::prelude::*;

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
        Ok(Solver(
            crate::Solver::new(program, query).map_err(|e| format!("{:?}", e))?,
        ))
    }

    #[wasm_bindgen]
    pub fn next(&mut self) -> Option<js_sys::Map> {
        self.0.next().map(solution_to_js)
    }

    #[wasm_bindgen]
    pub fn all(&mut self) -> js_sys::Array {
        self.0.by_ref().map(solution_to_js).collect()
    }
}

fn solution_to_js(solution: crate::Solution) -> js_sys::Map {
    let result = js_sys::Map::new();
    for (k, v) in solution {
        result.set(&k.into(), &v.into());
    }
    result
}
