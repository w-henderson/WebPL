mod builtins;
mod core;
mod error;
mod gc;
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

impl Solver {
    pub(crate) fn max_choice_point_stack_height(&self) -> usize {
        self.choice_points.capacity()
    }

    pub(crate) fn interrupt(interrupt: &std::sync::atomic::AtomicBool) {
        interrupt.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn check_interrupted(&self) -> Option<()> {
        if self
            .interrupt
            .compare_exchange(
                true,
                false,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            None // Force the interpreter to give control back to the test runner
        } else {
            Some(())
        }
    }
}
