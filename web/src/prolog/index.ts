export type Solution = Map<string, string>;

export type Error = {
  error: string;
  location?: {
    offset: number;
    line: number;
    column: number;
    query: boolean;
  }
};

export default abstract class Prolog {
  abstract name: string;

  abstract init(): Promise<void>;
  abstract solve(program: string, query: string): Promise<void>;
  abstract next(): Promise<Solution | undefined>;
  abstract all(): Promise<Solution[]>;

  handleError(e: any): string {
    return e.toString();
  }
}