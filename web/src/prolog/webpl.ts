import Prolog, { Solution, Error } from ".";
import init, { Solver } from "webpl";

export default class WebPL extends Prolog {
  private solver?: Solver;
  private ready: boolean = false;
  private gc: boolean = false;

  public name = "WebPL";

  public static with_gc(): WebPL {
    const webpl = new WebPL();
    webpl.gc = true;
    return webpl;
  }

  public async init(): Promise<void> {
    if (this.ready) return;
    await init();
    this.ready = true;
  }

  public async solve(program: string, query: string): Promise<void> {
    this.solver = this.gc ? Solver.new_with_gc(program, query) : new Solver(program, query);
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

  public handleError(e: Error): string {
    if (e.location) {
      return `Error in ${e.location!.query ? "query" : "program"} (${e.location!.line}:${e.location!.column}): ${e.error}`;
    } else {
      return e.error;
    }
  }
}