import TextContainer from "./TextContainer";

export default function Program(props: Readonly<{
  program: string,
  updateProgram: (p: string) => void,
  className?: string
}>) {
  return (
    <div className={props.className}>
      <TextContainer
        placeholder="Write your program here"
        text={props.program}
        update={props.updateProgram} />
    </div>
  )
}