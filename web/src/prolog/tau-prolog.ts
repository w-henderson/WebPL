import Prolog, { Solution } from ".";
import pl from "tau-prolog";

export default class TauProlog extends Prolog {
  private session?: any;

  public async init(): Promise<void> {
    if (this.session !== undefined) return;
    this.session = pl.create();
  }

  public async solve(program: string, query: string): Promise<void> {
    await new Promise((resolve, reject) => this!.session.consult(program, { success: resolve, error: reject }));
    await new Promise((resolve, reject) => this!.session.query(query, { success: resolve, error: reject }));
  }

  public async next(): Promise<Solution | undefined> {
    return new Promise((resolve, reject) => this!.session.answer({
      success: result => resolve(this.encodeSolution(result)),
      fail: () => resolve(undefined),
      error: reject,
      limit: reject
    }));
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

    for (const key in solution.links) {
      map.set(key, solution.links[key].toString());
    }

    return map;
  }
}