export default function Header(props: Readonly<{
  name: string,
  className?: string
}>) {
  return (
    <header className={props.className}>
      <h1>WebPL</h1>

      {props.name !== "WebPL" && (
        <span>Using {props.name}</span>
      )}
    </header>
  )
}