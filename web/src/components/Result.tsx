import styles from "./Result.module.css";

export default function Result(props: Readonly<{
  query: string,
  results: {
    map: Map<string, string>,
    duration?: number
  }[],
  complete: boolean
}>) {
  return (
    <div className={styles.result}>
      <div>{props.query}</div>
      <div>
        {props.results.map((result, i) => (
          <div key={i}>
            {result.map.size === 0 && (
              <div>True</div>
            )}

            <div>
              {Array.from(result.map.entries()).map(([key, value]) => `${key}: ${value}`).join(", ")}

              {result.duration && ` (${result.duration.toFixed(1)}ms)`}
            </div>
          </div>
        ))}

        {props.complete && props.results.length > 0 && (
          <div>No more results</div>
        )}

        {props.complete && props.results.length === 0 && (
          <div>No results</div>
        )}
      </div>
    </div>
  )
}