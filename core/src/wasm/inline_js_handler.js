/**
 * @param {string} js - The JavaScript code to evaluate
 * @param {Array} args - The arguments to pass to the JavaScript code
 * @param {(a: number, b: number) => bool} unify_wasm - WASM function to unify two variables, passed as pointers
 * @param {(a: string | number) => number} alloc_wasm - WASM function to allocate a new term
 * @returns {bool} - Whether the goal succeeded
 * 
 * @throws {string} - If the JavaScript code throws an error
 */
export function eval_js(js, args, unify_wasm, alloc_wasm) {
  const unify = (a, b) => {
    if (!("variable" in a)) throw new Error("Can only unify variables");
    if (!(typeof b === "number" || typeof b === "string")) throw new Error("Can only unify with numbers or strings");

    const a_ptr = a.variable;
    const b_ptr = alloc_wasm(b);

    return unify_wasm(a_ptr, b_ptr);
  };

  try {
    console.log(js);
    let fn = eval(js);
    return fn(...args);
  } catch (e) {
    throw e.toString();
  }
}