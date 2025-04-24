declare module "webpl" {
  export class Solver {
    /**
     * Sets up the solver with the given program and query.
     * Optionally uses garbage collection.
     * 
     * @param program The program source.
     * @param query The query to run.
     * @param gc Whether to enable garbage collection.
     * @returns A Promise that resolves to a Solver instance.
     */
    static solve(program: string, query: string, gc?: boolean): Promise<Solver>;

    /**
     * Gets the next result from the solver.
     * 
     * @returns A Promise resolving to the next result.
     */
    next(): Promise<Map<string, string> | undefined>;

    /**
     * Gets all results from the solver.
     * 
     * @returns A Promise resolving to all results.
     */
    all(): Promise<Map<string, string>[]>;
  }

  /**
   * Initializes the module, optionally enabling a web worker.
   * 
   * @param enableWebWorker Whether to use a web worker.
   * @returns A Promise that resolves when initialization is complete.
   */
  const init: (enableWebWorker?: boolean) => Promise<void>;

  export default init;
}
