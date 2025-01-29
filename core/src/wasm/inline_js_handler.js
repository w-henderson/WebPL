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