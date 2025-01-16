import Prolog, { Solution } from ".";
import init, { Solver } from "webpl";

export default class WebPL extends Prolog {
  private solver?: Solver;
  private ready: boolean = false;

  public name = "WebPL";

  public async init(): Promise<void> {
    if (this.ready) return;
    await init();
    this.ready = true;
  }

  public async solve(program: string, query: string): Promise<void> {
    this.solver = new Solver(program, query);
  }

  public async next(): Promise<Solution | undefined> {
    const solution = this.solver?.next();
    if (solution === undefined) return undefined;
    return solution;
  }

  public async all(): Promise<Solution[]> {
    const solutions = this.solver?.all();
    if (solutions === undefined) return [];
    return solutions;
  }
}