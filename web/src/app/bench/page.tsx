"use client"

import styles from "./bench.module.css";

import { useState, useEffect } from "react";

import { run } from "webpl-bench";

import WebPL from "@/prolog/webpl";
import WebPLGC from "@/prolog/webpl-gc";
import SWIPL from "@/prolog/swipl";
import SWIPLExperimental from "@/prolog/swipl-experimental";
import TreallaProlog from "@/prolog/trealla-prolog";
import TauProlog from "@/prolog/tau-prolog";

const WEBPL = new WebPL();
const WEBPL_GC = new WebPLGC();
let SWIPL_ = new SWIPL();
let SWIPL_EXPERIMENTAL = new SWIPLExperimental();
const TREALLA_PROLOG = new TreallaProlog();
const TAU_PROLOG = new TauProlog();

const ENGINES = [
  "WebPL",
  "WebPL (GC)",
  "SWI-Prolog",
  "SWI (Experimental)",
  "Trealla Prolog",
  "Tau Prolog"
];

export default function BenchPage() {
  const [log, setLog] = useState<string>("");
  const [state, setState] = useState<"idle" | "running" | "complete">("idle");
  const [results, setResults] = useState<any>(null);
  const [enabledEngines, setEnabledEngines] = useState<string[]>([
    "WebPL",
    "WebPL (GC)",
    "SWI-Prolog",
    "Trealla Prolog",
  ]);

  useEffect(() => {
    (async () => {
      await WEBPL.init();
      await WEBPL_GC.init();
      await SWIPL_.init();
      await SWIPL_EXPERIMENTAL.init();
      await TREALLA_PROLOG.init();
      await TAU_PROLOG.init();
    })();
  }, []);

  // scroll textarea to bottom
  useEffect(() => {
    const textarea = document.querySelector("textarea")!;
    textarea.scrollTop = textarea.scrollHeight;
  }, [log]);

  const startBenchmark = async () => {
    setState("running");

    const results: any = {};

    for (const engine of [WEBPL, WEBPL_GC]) {
      if (!enabledEngines.includes(engine.name)) continue;
      results[engine.name] = await run(engine.name, {
        solve: async (program: string, query: string) => {
          await engine.solve(program, query.slice(0, -1) + ", statistics(allocated, Mem).");
          const result = await engine.next();
          return {
            ok: result !== undefined,
            memory: result ? parseInt(result.get("Mem")!) : undefined
          }
        },
        log: s => setLog(log => log + s),
        clean: undefined
      });
    }

    for (const engine of [SWIPL_, SWIPL_EXPERIMENTAL]) {
      if (!enabledEngines.includes(engine.name)) continue;
      results[engine.name] = await run(engine.name, {
        solve: async (program: string, query: string) => {
          await engine.solve(program, query.slice(0, -1) + ", statistics(stack, Mem).");
          const result = await engine.next();
          return {
            ok: result !== undefined,
            memory: result ? parseInt(result.get("Mem")!) : undefined
          }
        },
        log: s => setLog(log => log + s),
        clean: async () => {
          if (engine.name === "SWI-Prolog") {
            SWIPL_ = new SWIPL();
            await SWIPL_.init();
          } else {
            SWIPL_EXPERIMENTAL = new SWIPLExperimental();
            await SWIPL_EXPERIMENTAL.init();
          }
        }
      });
    }

    for (const engine of [TREALLA_PROLOG, TAU_PROLOG]) {
      if (!enabledEngines.includes(engine.name)) continue;
      results[engine.name] = await run(engine.name, {
        solve: async (program: string, query: string) => {
          await engine.solve(program, query);
          return {
            ok: (await engine.all()).length > 0,
            memory: undefined as number | undefined
          }
        },
        log: s => setLog(log => log + s),
        clean: undefined,
      });
    }

    setState("complete");
    setResults(results);
  };

  const getResults = () => {
    const blob = new Blob([JSON.stringify(results)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "results.json";
    a.click();
  };

  return (
    <main className={styles.container}>
      <h1>Benchmark</h1>

      {state === "idle" && <span onClick={startBenchmark}>Start Benchmark</span>}
      {state === "running" && <span className={styles.running}>Running...</span>}
      {state === "complete" && <span onClick={getResults}>Download Results</span>}

      <div>
        {ENGINES.map(engine => (
          <label key={engine}>
            <input
              type="checkbox"
              checked={enabledEngines.includes(engine)}
              onChange={e => setEnabledEngines(enabledEngines => {
                if (e.target.checked) return [...enabledEngines, engine];
                else return enabledEngines.filter(e => e !== engine);
              })} />
            {engine}
          </label>
        ))}
      </div>

      <textarea
        value={log}
        placeholder="Output will appear here"
        readOnly={true} />
    </main>
  );
}