use crate::WebPL;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn solve(program: &str, query: &str) -> Vec<String> {
    let mut webpl = WebPL::new(program).unwrap();
    let solver = webpl.solve(query).unwrap();
    solver.map(|x| format!("{}={}", x[0].0, x[0].1)).collect()
}
