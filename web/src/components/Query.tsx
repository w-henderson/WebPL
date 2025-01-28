import TextContainer from "./TextContainer";

import { Search, FastArrowRight, NavArrowRight, Settings } from "iconoir-react";
import styles from "./Loader.module.css";

export default function Query(props: Readonly<{
  query: string,
  updateQuery: (q: string) => void,
  loading: boolean,
  settingsOpen: boolean,
  setSettingsOpen: (open: boolean) => void,
  solve: () => void,
  one: () => void,
  all: () => void,
  className?: string
}>) {
  return (
    <div className={props.className}>
      <div>
        <TextContainer
          placeholder="Write your query here"
          text={props.query}
          update={props.updateQuery}
          scrollable={true} />
      </div>

      <div>
        {props.loading ? (
          <div className={styles.loaderContainer}>
            <span className={styles.loader} />
          </div>
        ) : <Search width={32} height={32} onClick={props.solve} />}
        <NavArrowRight width={32} height={32} onClick={props.one} />
        <FastArrowRight width={32} height={32} onClick={props.all} />
        <Settings width={32} height={32} style={{ marginTop: "auto" }}
          onClick={() => props.setSettingsOpen(!props.settingsOpen)} />
      </div>
    </div>
  )
}