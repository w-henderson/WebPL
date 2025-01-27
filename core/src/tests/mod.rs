mod builtins;
mod core;
mod error;
mod lco;

use crate::Solver;

pub type SolverFn = fn(program: &str, query: &str) -> Solver;

#[macro_export]
macro_rules! test {
    ($name:ident, $fn:expr) => {
        #[test]
        fn $name() {
            $fn(|program, query| Solver::new(program, query).unwrap());
            $fn(|program, query| Solver::new_with_gc(program, query).unwrap());
        }
    };

    (LONG $name:ident, $fn:expr) => {
        #[test]
        #[ignore = "Long in debug"]
        fn $name() {
            $fn(|program, query| Solver::new(program, query).unwrap());
            $fn(|program, query| Solver::new_with_gc(program, query).unwrap());
        }
    };
}
