import initWasm, { Solver as SolverWasm } from "./wasm";

/**
 * The worker that will run everything in the background.
 * @type {Worker}
 */
let worker;

/**
 * The Solver instance (if the worker is not used).
 * @type {SolverWasm}
 */
let solver;

let useWorker;

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
    if (useWorker) {
      await post("solve", { program, query, gc });
      return new Solver();
    } else {
      solver = gc ? SolverWasm.new_with_gc(program, query) : new SolverWasm(program, query);
      return new Solver();
    }
  }

  next() {
    return useWorker ? post("next") : Promise.resolve(solver.next());
  }

  all() {
    return useWorker ? post("all") : Promise.resolve(solver.all());
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

export default async (enableWebWorker = true) => {
  useWorker = enableWebWorker;

  if (enableWebWorker) {
    worker = new Worker(new URL("./worker.js", import.meta.url));
    worker.onmessage = e => recv(e.data);
    await post("init");
  } else {
    await initWasm();
  }
};