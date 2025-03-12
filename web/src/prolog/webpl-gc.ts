import { Solver } from "webpl";
import WebPL from "./webpl";

export default class WebPLGC extends WebPL {
  public name: string = "WebPL (GC)";

  public async solve(program: string, query: string): Promise<void> {
    this.solver = await Solver.solve(program, query, true);
  }
}