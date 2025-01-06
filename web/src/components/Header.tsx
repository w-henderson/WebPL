export default function Header(props: Readonly<{ className?: string }>) {
  return (
    <header className={props.className}>
      <h1>WebPL</h1>
    </header>
  )
}