import Prolog, { Solution } from ".";
import { load, Prolog as TreallaWasm } from "trealla";

export default class TreallaProlog extends Prolog {
  private tpl?: TreallaWasm;
  private query?: any;

  public async init(): Promise<void> {
    if (this.tpl !== undefined) return;
    this.tpl = new TreallaWasm();
  }

  public async solve(program: string, query: string): Promise<void> {
    await this.tpl!.consultText(program);
    this.query = this.tpl!.query(query);
  }

  public async next(): Promise<Solution | undefined> {
    const solution: any = await this.query!.next();
    if (solution.done || solution.value.status === "failure") return undefined;
    return this.encodeSolution(solution);
  }

  public async all(): Promise<Solution[]> {
    const solutions: Solution[] = [];
    let solution: Solution | undefined = await this.next();
    while (solution) {
      solutions.push(solution);
      solution = await this.next();
    }
    return solutions;
  }

  private encodeSolution(solution: any): Solution {
    const map: Map<string, string> = new Map();

    for (const key in solution.value.answer) {
      map.set(key, JSON.stringify(solution.value.answer[key]));
    }

    return map;
  }
}