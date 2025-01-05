import Prolog, { Solution } from ".";
import init, { Solver } from "webpl";

export default class WebPL extends Prolog {
  private solver?: Solver;

  public async init(): Promise<void> {
    if (this.ready) return;

    return new Promise(resolve => {
      init();
      this.ready = true;
      resolve();
    });
  }

  public solve(program: string, query: string): void {
    this.solver = new Solver(program, query);
  }

  public next(): Promise<Solution | undefined> {
    return new Promise(resolve => {
      const solution = this.solver?.next();
      if (solution === undefined) return resolve(undefined);
      resolve(solution);
    });
  }

  public all(): Promise<Solution[]> {
    return new Promise(resolve => {
      const solutions = this.solver?.all();
      if (solutions === undefined) return resolve([]);
      resolve(solutions);
    });
  }
}