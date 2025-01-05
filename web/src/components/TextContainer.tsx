import styles from "./TextContainer.module.css";

export default function TextContainer(props: Readonly<{ placeholder: string, text: string, update: (x: string) => void }>) {
  return (
    <textarea
      className={styles.textarea}
      placeholder={props.placeholder}
      content={props.text}
      onChange={e => props.update(e.target.value)}
    />
  );
}