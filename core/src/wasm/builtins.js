/**
 * Unifies two terms.
 * @param {HeapTermPtr} a
 * @param {HeapTermPtr} b 
 * @returns {bool} Whether the unification was successful.
 */
export function unify(a, b) {
  if (Array.isArray(a)) a = list(...a);
  if (Array.isArray(b)) b = list(...b);

  const a_ptr = this.alloc_wasm(a);
  const b_ptr = this.alloc_wasm(b);

  return this.unify_wasm(a_ptr, b_ptr);
}

/**
 * Makes a synchronous HTTP request.
 * @param {string} url
 * @param {string} method
 * @returns {string} The response text
 */
export function fetch(url, method = "GET") {
  const xhr = new XMLHttpRequest();
  xhr.open(method, url, false);
  xhr.send();
  return xhr.responseText;
}

/**
 * Constructs a compound term.
 * @param {string} functor
 * @param  {...any} args
 */
export function compound(functor, ...args) {
  return { functor, args }
}

/**
 * Constructs a list term.
 * @param  {...any} args
 */
export function list(...args) {
  let result = "[]";
  for (let i = args.length - 1; i >= 0; i--) {
    result = compound(".", args[i], result);
  }
  return result;
}