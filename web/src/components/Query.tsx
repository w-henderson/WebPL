import TextContainer from "./TextContainer";

import { Search, FastArrowRight, NavArrowRight, Settings } from "iconoir-react";

export default function Query(props: Readonly<{
  query: string,
  updateQuery: (q: string) => void,
  solve: () => void,
  one: () => void,
  all: () => void,
  className?: string
}>) {
  return (
    <div className={props.className}>
      <TextContainer
        placeholder="Write your query here"
        text={props.query}
        update={props.updateQuery} />

      <div>
        <Search width={32} height={32} onClick={props.solve} />
        <NavArrowRight width={32} height={32} onClick={props.one} />
        <FastArrowRight width={32} height={32} onClick={props.all} />
        <Settings width={32} height={32} style={{ marginTop: "auto" }} />
      </div>
    </div>
  )
}