import benchmarks from "./dist/benchmarks";

const TARGET_TIME_PER_BENCHMARK = 5000;
const MIN_ITERS = 10;
const MAX_ITERS = 1000;

/**
 * @param {async () => void} fn
 */
async function warmup(fn) {
  let ok = true;

  let start = performance.now();
  ok &= await fn();
  let end = performance.now();

  if (end - start < 100) {
    for (let i = 0; i < 20; i++) {
      ok &= await fn();
    }
  }

  if (!ok) {
    throw new Error("Not enough solutions");
  }
}

/**
 * @param {string} name
 * @param {async (program: string, query: string) => bool} solve
 * @param {(s: string) => void} log
 */
export async function run(solverName, solve, log) {
  let results = [];

  log(`=== ${solverName} ===\n`);

  for (const name in benchmarks) {
    try {
      log(`${name}... `);

      let fn = async () => await solve(benchmarks[name], "top.");
      await warmup(fn);
      let samples = [];

      let benchmarkStart = performance.now();
      for (let i = 0; i < MAX_ITERS; i++) {
        let start = performance.now();
        await fn();
        let end = performance.now();

        samples.push(end - start);

        if (end - benchmarkStart > TARGET_TIME_PER_BENCHMARK && i > MIN_ITERS) {
          break;
        }
      }

      let avg = Math.round((samples.reduce((a, b) => a + b, 0) / samples.length) * 100) / 100;
      log(`${avg}ms\n`);

      results.push({ solverName, samples });
    } catch (e) {
      log(`error: ${e.toString()}\n`);

      results.push({ solverName, samples: [] });
    }
  }

  log("Complete\n\n");

  return results;
}

export default benchmarks;