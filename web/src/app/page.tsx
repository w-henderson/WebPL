"use client"

import { useState, useEffect } from "react";

import styles from "./page.module.css";

import Program from "@/components/Program";
import Results from "@/components/Results";
import Query from "@/components/Query";
import Header from "@/components/Header";

import Prolog from "@/prolog";
import WebPL from "@/prolog/webpl";
import SWIPL from "@/prolog/swipl";
import TauProlog from "@/prolog/tau-prolog";
import TreallaProlog from "@/prolog/trealla-prolog";

type QueryResults = {
  query: string,
  bindings: Map<string, string>[],
  complete: boolean
}

export default function Home() {
  const [prolog, setProlog] = useState<Prolog>(new TreallaProlog());
  const [program, setProgram] = useState<string>("");
  const [query, setQuery] = useState<string>("");
  const [results, setResults] = useState<QueryResults[]>([]);

  useEffect(() => {
    prolog.init();
  }, [prolog]);

  const appendResult = (complete: boolean, ...solutions: Map<string, string>[]) => {
    setResults(prevResults => {
      const lastResult = prevResults[prevResults.length - 1];
      return prevResults.slice(0, prevResults.length - 1).concat({
        ...lastResult,
        bindings: lastResult.bindings.concat(solutions),
        complete: lastResult.complete || complete
      });
    });
  };

  const completeResults = () => {
    setResults(prevResults => {
      const lastResult = prevResults[prevResults.length - 1];
      return prevResults.slice(0, prevResults.length - 1).concat({
        ...lastResult,
        complete: true
      });
    });
  };

  return (
    <main className={styles.container}>
      <Header className={styles.header} />

      <Program
        className={styles.program}
        program={program}
        updateProgram={setProgram} />

      <Results
        className={styles.results}
        results={results} />

      <Query
        className={styles.query}
        query={query}
        updateQuery={setQuery}
        solve={async () => {
          await prolog.solve(program, query);
          setResults(prevResults => [...prevResults, { query, bindings: [], complete: false }]);
          const solution = await prolog.next();
          if (solution) appendResult(false, solution);
          else completeResults();
        }}
        one={async () => {
          if (results.length > 0 && results[results.length - 1].query === query) {
            const solution = await prolog.next();
            if (solution) appendResult(false, solution);
            else completeResults();
          }
        }}
        all={async () => {
          if (results.length > 0 && results[results.length - 1].query === query) {
            const solutions = await prolog.all();
            if (solutions) appendResult(true, ...solutions);
            else completeResults();
          }
        }} />
    </main>
  );
}
