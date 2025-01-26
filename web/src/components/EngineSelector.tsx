import Prolog from "@/prolog";

import styles from "./EngineSelector.module.css";

import WebPL from "@/prolog/webpl";
import SWIPL from "@/prolog/swipl";
import TreallaProlog from "@/prolog/trealla-prolog";
import TauProlog from "@/prolog/tau-prolog";
import { InfoCircle } from "iconoir-react";

const engines = [
  { name: "WebPL", engine: () => new WebPL() },
  { name: "WebPL (with GC)", engine: () => WebPL.with_gc() },
  { name: "SWI-Prolog", engine: () => new SWIPL() },
  { name: "Trealla Prolog", engine: () => new TreallaProlog() },
  { name: "Tau Prolog", engine: () => new TauProlog() }
];

export default function EngineSelector(props: Readonly<{
  prolog: Prolog,
  setProlog: (prolog: Prolog) => void,
  open: boolean,
  setOpen: (open: boolean) => void
}>) {
  return (
    <div className={styles.container}>
      {props.open && engines.map(({ name, engine }) => (
        <div
          key={name}
          className={props.prolog.name === name ? styles.selected : undefined}
          onClick={() => {
            props.setOpen(false);
            props.setProlog(engine());
          }}>
          {name}
        </div>
      ))}

      {props.open && (
        <span>
          <InfoCircle width={14} height={14} />
          WebPL
          <code onClick={() => open(`https://github.com/w-henderson/WebPL/commit/${process.env.NEXT_PUBLIC_GIT_COMMIT_HASH}`, "_blank")}>
            {process.env.NEXT_PUBLIC_GIT_COMMIT_HASH}
          </code>
        </span>
      )}
    </div>
  )
}