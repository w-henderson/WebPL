import styles from "./Result.module.css";

export default function Result(props: Readonly<{
  query: string,
  results: Map<string, string>[],
  complete: boolean
}>) {
  return (
    <div className={styles.result}>
      <div>{props.query}</div>
      <div>
        {props.results.map((result, i) => (
          <div key={i}>
            {Array.from(result.entries()).map(([key, value]) => (
              <div key={key}>{key}: {value}</div>
            ))}
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