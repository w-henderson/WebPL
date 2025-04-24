import init, { Solver } from "./wasm/webpl.js";

/**
 * @type {Solver}
 */
let solver;

function ok(id, data = undefined) {
  postMessage({ id, ok: true, data });
}

onmessage = async e => {
  let { id, fn, data } = e.data;

  try {
    switch (fn) {
      case "init":
        await init();
        ok(id);
        break;
      case "solve":
        solver = data.gc ? Solver.new_with_gc(data.program, data.query)
          : new Solver(data.program, data.query);
        ok(id);
        break;
      case "next":
        ok(id, solver.next());
        break;
      case "all":
        ok(id, solver.all());
        break;
    }
  } catch (e) {
    postMessage({ id, ok: false, data: e });
  }
};