import Prolog from "@/prolog";

import styles from "./EngineSelector.module.css";

import WebPL from "@/prolog/webpl";
import SWIPL from "@/prolog/swipl";
import TreallaProlog from "@/prolog/trealla-prolog";
import TauProlog from "@/prolog/tau-prolog";

const engines = [
  { name: "WebPL", engine: WebPL },
  { name: "SWI-Prolog", engine: SWIPL },
  { name: "Trealla Prolog", engine: TreallaProlog },
  { name: "Tau Prolog", engine: TauProlog }
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
            props.setProlog(new engine());
          }}>
          {name}
        </div>
      ))}
    </div>
  )
}