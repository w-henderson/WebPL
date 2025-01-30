"use client"

import styles from "./bench.module.css";

import { useState, useEffect } from "react";

import { run } from "webpl-bench";

import WebPL from "@/prolog/webpl";
import WebPLGC from "@/prolog/webpl-gc";
import SWIPL from "@/prolog/swipl";
import TreallaProlog from "@/prolog/trealla-prolog";
import TauProlog from "@/prolog/tau-prolog";

const WEBPL = new WebPL();
const WEBPL_GC = new WebPLGC();
const SWIPL_ = new SWIPL();
const TREALLA_PROLOG = new TreallaProlog();
const TAU_PROLOG = new TauProlog();

export default function BenchPage() {
  const [log, setLog] = useState<string>("");
  const [state, setState] = useState<"idle" | "running" | "complete">("idle");
  const [results, setResults] = useState<any>(null);

  useEffect(() => {
    (async () => {
      await WEBPL.init();
      await WEBPL_GC.init();
      await SWIPL_.init();
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

    for (const engine of [WEBPL, WEBPL_GC, SWIPL_, TREALLA_PROLOG, TAU_PROLOG]) {
      results[engine.name] = await run(engine.name, async (program: string, query: string) => {
        await engine.solve(program, query);
        return (await engine.all()).length > 0;
      }, s => setLog(log => log + s));
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

      <textarea
        value={log}
        placeholder="Output will appear here"
        readOnly={true} />
    </main>
  );
}