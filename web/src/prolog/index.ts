export type Solution = Map<string, string>;

export default abstract class Prolog {
  abstract name: string;

  abstract init(): Promise<void>;
  abstract solve(program: string, query: string): Promise<void>;
  abstract next(): Promise<Solution | undefined>;
  abstract all(): Promise<Solution[]>;
}