import * as builtins from "./builtins.js";

/**
 * @param {string} js - The JavaScript code to evaluate
 * @param {Array} args - The arguments to pass to the JavaScript code
 * @param {(a: number, b: number) => bool} unify_wasm - WASM function to unify two variables, passed as pointers
 * @param {(a: string | number) => number} alloc_wasm - WASM function to allocate a new term
 * @returns {bool} - Whether the goal succeeded
 * 
 * @throws {string} - If the JavaScript code throws an error
 */
export function eval_js(js, arg_names, arg_values, unify_wasm, alloc_wasm) {
  const env = { unify_wasm, alloc_wasm };
  const builtin_names = Object.keys(builtins);
  const builtin_values = Object.values(builtins).map(fn => fn.bind(env));

  try {
    let fn = new Function(...builtin_names, ...arg_names, js)
      .bind(globalThis, ...builtin_values);
    return fn(...arg_values) !== false;
  } catch (e) {
    throw e.toString();
  }
}