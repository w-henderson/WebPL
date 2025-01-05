export type Solution = Map<string, string>;

export default abstract class Prolog {
  public ready: boolean = false;

  abstract init(): Promise<void>;
  abstract solve(program: string, query: string): void;
  abstract next(): Promise<Solution | undefined>;
  abstract all(): Promise<Solution[]>;
}