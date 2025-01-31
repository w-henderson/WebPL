import Prolog, { Solution, Error } from ".";
import init, { Solver } from "webpl";

export default class WebPL extends Prolog {
  protected solver?: Solver;
  private ready: boolean = false;

  public name = "WebPL";

  public async init(): Promise<void> {
    if (this.ready) return;
    await init();
    this.ready = true;
  }

  public async solve(program: string, query: string): Promise<void> {
    this.solver = await Solver.solve(program, query);
  }

  public async next(): Promise<Solution | undefined> {
    const solution = await this.solver?.next();
    if (solution === undefined) return undefined;
    return solution;
  }

  public async all(): Promise<Solution[]> {
    const solutions = await this.solver?.all();
    if (solutions === undefined) return [];
    return solutions;
  }

  public handleError(e: Error): string {
    if (e.location) {
      return `Error in ${e.location!.query ? "query" : "program"} (${e.location!.line}:${e.location!.column}): ${e.error}`;
    } else {
      return e.error;
    }
  }
}