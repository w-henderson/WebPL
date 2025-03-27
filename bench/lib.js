import benchmarks from "./dist/benchmarks";

const TARGET_TIME_PER_BENCHMARK = 5000;
const MIN_ITERS = 10;
const MAX_ITERS = 1000;

/**
 * @param {() => Promise<void>} fn
 */
async function warmup(fn) {
  let ok = true;

  let start = performance.now();
  ok &= (await fn()).ok;
  let end = performance.now();

  if (end - start < 100) {
    for (let i = 0; i < 20; i++) {
      ok &= (await fn()).ok;
    }
  }

  if (!ok) {
    throw new Error("Not enough solutions");
  }
}

/**
 * @param {string} name
 * @param { {
 *   solve: (program: string, query: string) => Promise<{ ok: boolean, memory: number | undefined }>,
 *   clean: (() => Promise<void>) | undefined,
 *   log: (s: string) => void
 * }} fns
 */
export async function run(solverName, fns) {
  let results = [];

  fns.log(`=== ${solverName} ===\n`);

  for (const name in benchmarks) {
    try {
      fns.log(`${name}...`);

      if (fns.clean) await fns.clean();
      let fn = async () => await fns.solve(benchmarks[name], "top.");
      await warmup(fn);
      let timeSamples = [];
      let memorySamples = [];

      fns.log(" ... ");

      let benchmarkStart = performance.now();
      for (let i = 0; i < MAX_ITERS; i++) {
        let start = performance.now();
        await fn();
        let end = performance.now();

        timeSamples.push(end - start);

        if (end - benchmarkStart > TARGET_TIME_PER_BENCHMARK && i > MIN_ITERS) {
          break;
        }
      }

      if (fns.clean) await fns.clean();
      let memory = (await fn()).memory;
      if (memory) memorySamples.push(memory);

      let avgTime = Math.round((timeSamples.reduce((a, b) => a + b, 0) / timeSamples.length) * 100) / 100;
      if (memory) {
        fns.log(`${avgTime}ms, ${memory} bytes\n`);
      } else {
        fns.log(`${avgTime}ms\n`);
      }

      results.push({
        name,
        timeSamples,
        memorySamples
      });
    } catch (e) {
      fns.log(`error: ${e.toString()}\n`);
      console.error(e);

      results.push({ name, samples: [] });
    }
  }

  fns.log("Complete\n\n");

  return results;
}

export default benchmarks;