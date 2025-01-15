"use client"

import { useState, useEffect } from "react";

import styles from "./page.module.css";

import Program from "@/components/Program";
import Results from "@/components/Results";
import Query from "@/components/Query";
import Header from "@/components/Header";

import Prolog from "@/prolog";
import WebPL from "@/prolog/webpl";
import EngineSelector from "@/components/EngineSelector";

type QueryResults = {
  query: string,
  bindings: Map<string, string>[],
  complete: boolean
}

export default function Home() {
  const [prolog, setProlog] = useState<Prolog>(new WebPL());
  const [loading, setLoading] = useState<boolean>(true);
  const [program, setProgram] = useState<string>("");
  const [query, setQuery] = useState<string>("");
  const [results, setResults] = useState<QueryResults[]>([]);
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);

  useEffect(() => {
    prolog.init();
    setLoading(false);
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
        loading={loading}
        settingsOpen={settingsOpen}
        setSettingsOpen={setSettingsOpen}
        solve={async () => {
          await prolog.solve(program, query);
          setResults(prevResults => [...prevResults, { query, bindings: [], complete: false }]);
          setLoading(true);
          const solution = await prolog.next();
          setLoading(false);
          if (solution) appendResult(false, solution);
          else completeResults();
        }}
        one={async () => {
          if (results.length > 0 && results[results.length - 1].query === query) {
            setLoading(true);
            const solution = await prolog.next();
            setLoading(false);
            if (solution) appendResult(false, solution);
            else completeResults();
          }
        }}
        all={async () => {
          if (results.length > 0 && results[results.length - 1].query === query) {
            setLoading(true);
            const solutions = await prolog.all();
            setLoading(false);
            if (solutions) appendResult(true, ...solutions);
            else completeResults();
          }
        }} />

      <EngineSelector
        prolog={prolog}
        setProlog={engine => {
          setLoading(true);
          setResults([]);
          setProlog(engine);
        }}
        open={settingsOpen}
        setOpen={setSettingsOpen} />
    </main>
  );
}
