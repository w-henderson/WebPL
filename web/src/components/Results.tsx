import React, { useEffect, useRef } from "react";
import Result from "./Result";

export default function Results(props: Readonly<{
  className?: string,
  results: {
    query: string,
    bindings: Map<string, string>[],
    complete: boolean
  }[]
}>) {
  const resultsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (resultsEndRef.current) {
      resultsEndRef.current.scrollIntoView();
    }
  }, [props.results]);

  if (props.results.length === 0) {
    return (
      <div className={props.className}>
        Results will appear here!
      </div>
    )
  }

  return (
    <div className={props.className}>
      {props.results.map((result, i) => (
        <Result
          key={i}
          query={result.query}
          results={result.bindings}
          complete={result.complete} />
      ))}
      <div ref={resultsEndRef} />
    </div>
  )
}