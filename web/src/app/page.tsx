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
  bindings: {
    map: Map<string, string>,
    duration?: number
  }[],
  complete: boolean,
  error?: string
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

  const appendResult = (complete: boolean, ...solutions: {
    map: Map<string, string>,
    duration?: number
  }[]) => {
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

  const errorResult = (e: any) => {
    setLoading(false);
    setResults(prevResults => {
      const lastResult = prevResults[prevResults.length - 1];
      return prevResults.slice(0, prevResults.length - 1).concat({
        ...lastResult,
        bindings: [],
        complete: true,
        error: prolog.handleError(e)
      });
    });
  };

  const solve = async (solveAll: boolean) => {
    try {
      await prolog.solve(program, query);
      setResults(prevResults => [...prevResults, { query, bindings: [], complete: false }]);
    } catch (e: any) {
      setResults(prevResults => [...prevResults, {
        query,
        bindings: [],
        complete: true,
        error: prolog.handleError(e)
      }]);
      return;
    }

    if (solveAll) await all();
    else await one();
  };

  const one = async () => {
    try {
      setLoading(true);
      const start = performance.now();
      const solution = await prolog.next();
      const end = performance.now();
      setLoading(false);

      if (solution) {
        appendResult(false, {
          map: solution,
          duration: end - start
        });
      } else {
        completeResults();
      }
    } catch (e: any) {
      errorResult(e);
    }
  };

  const all = async () => {
    try {
      setLoading(true);
      const start = performance.now();
      const solutions = await prolog.all();
      const end = performance.now();
      setLoading(false);

      if (solutions && solutions.length > 0) {
        const newSolutions: {
          map: Map<string, string>,
          duration?: number
        }[] = solutions.map(solution => ({
          map: solution
        }));
        newSolutions[newSolutions.length - 1].duration = end - start;
        appendResult(true, ...newSolutions)
      } else {
        completeResults();
      }
    } catch (e: any) {
      errorResult(e);
    }
  };

  return (
    <main className={styles.container}>
      <Header
        className={styles.header}
        name={prolog.name} />

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
        solve={() => solve(false)}
        one={() => {
          if (results.length > 0
            && results[results.length - 1].query === query
            && !results[results.length - 1].complete) {
            one();
          } else {
            solve(false);
          }
        }}
        all={() => {
          if (results.length > 0
            && results[results.length - 1].query === query
            && !results[results.length - 1].complete) {
            all();
          } else {
            solve(true);
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
