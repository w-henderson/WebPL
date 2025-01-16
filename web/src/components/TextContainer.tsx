import Editor from "react-simple-code-editor";
import { highlight, languages } from "prismjs/components/prism-core";
import "prismjs/themes/prism.css";
import "prismjs/components/prism-prolog";

import styles from "./TextContainer.module.css";

export default function TextContainer(props: Readonly<{ placeholder: string, text: string, update: (x: string) => void }>) {
  return (
    <Editor
      value={props.text}
      onValueChange={code => props.update(code)}
      highlight={code => highlight(code, languages.prolog)}
      className={styles.editor}
      placeholder={props.placeholder}
      padding={16}
    />
  );
}