import Prolog, { Solution } from ".";
import SWIPLWasm from "swipl-wasm";

export default class SWIPL extends Prolog {
  private swipl?: any;
  private query?: any;

  public async init(): Promise<void> {
    if (this.swipl !== undefined) return;
    this.swipl = await SWIPLWasm({ arguments: ["-q"] });
  }

  public async solve(program: string, query: string) {
    if (this.query) await this.query!.close();
    await this.swipl!.prolog.load_string(program, "input.pl");
    this.query = this.swipl!.prolog.query(query);
  }

  public async next(): Promise<Solution | undefined> {
    const solution: any = this.query!.next();
    if (solution.done) return undefined;
    return this.encodeSolution(solution);
  }

  public async all(): Promise<Solution[]> {
    const solutions: Solution[] = [];
    let solution: any = this.query!.next();
    do {
      solutions.push(this.encodeSolution(solution)!);
      solution = this.query!.next();
    } while (!solution.done);
    return solutions;
  }

  private encodeSolution(solution: any): Solution | undefined {
    if (solution === undefined) return undefined;

    const map: Map<string, string> = new Map();

    for (const key in solution.value) {
      if (key !== "$tag") {
        map.set(key, JSON.stringify(solution.value[key]));
      }
    }

    return map;
  }
}