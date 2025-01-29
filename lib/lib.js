/**
 * The worker that will run everything in the background.
 * @type {Worker}
 */
let worker;

let pending = new Map();
let nextId = 0;

export class Solver {
  /**
   * Sets up the solver.
   * @param {string} program
   * @param {string} query
   * @returns {Promise<Solver>}
   */x
  static async solve(program, query, gc = false) {
    await post("solve", { program, query, gc });
    return new Solver();
  }

  next() {
    return post("next");
  }

  all() {
    return post("all");
  }
}

/**
 * Sends a message to the worker and returns a Promise that resolves when the worker responds.
 * @param {string} fn
 * @param {any} data 
 * @returns {Promise<any>}
 */
function post(fn, data) {
  let id = nextId++;
  let result = new Promise((res, rej) => pending.set(id, { res, rej }));
  worker.postMessage({ id, fn, data });
  return result;
}

/**
 * Handles responses from the worker.
 * @param {{ id: number, ok: boolean, data: any }}
 */
function recv({ id, ok, data }) {
  let { res, rej } = pending.get(id);
  if (ok) res(data);
  else rej(data);
  pending.delete(id);
}

export default async () => {
  worker = new Worker(new URL("./worker.js", import.meta.url));
  worker.onmessage = e => recv(e.data);
  await post("init");
};