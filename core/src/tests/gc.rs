use crate::gc::GarbageCollector;
use crate::Solver;

use std::time::Duration;

fn test_constant_memory(program: &str, query: &str) {
    let mut solver = Solver::new_with_gc(program, query).unwrap();

    // Warm up
    run_for(&mut solver, Duration::from_secs(1));
    GarbageCollector::run(&mut solver);
    let initial_memory = solver.heap.size();

    // Run some more
    run_for(&mut solver, Duration::from_secs(1));

    // Check that the memory usage is roughly the same
    for _ in 0..10 {
        run_for(&mut solver, Duration::from_millis(100));
        GarbageCollector::run(&mut solver);
        let memory = solver.heap.size();

        if memory <= initial_memory {
            return;
        }
    }

    // If we get here, the memory usage increased
    panic!(
        "Memory usage increased: {} -> {}",
        initial_memory,
        solver.heap.size()
    );
}

fn run_for(solver: &mut Solver, time: Duration) {
    let interrupt = solver.interrupt.clone();
    std::thread::spawn(move || {
        std::thread::sleep(time);
        Solver::interrupt(interrupt.as_ref());
    });
    solver.next();
}

// Wielemaker and Neumerkel, Precise Garbage Collection in Prolog (2008)
#[test]
#[ignore = "Long in debug"]
fn precise_gc_1() {
    test_constant_memory(
        r#"
            run :- run(_).
            run(X) :- freeze(X, dummy(X)), X = 1, run(T).
            dummy(_).
        "#,
        "run.",
    );
}

#[test]
#[ignore = "Long in debug"]
fn precise_gc_2() {
    test_constant_memory(
        r#"
            run :- run(_,_).
            run(L0, L) :- f(L0, L1), dummy(L1, L).
            f([g|X], Y) :- f(X, Y).
            dummy(Xs, Xs).
        "#,
        "run.",
    );
}

#[test]
#[ignore = "Long in debug"]
fn precise_gc_3() {
    test_constant_memory(
        r#"
            run :- run(_,_).
            run(L0, L) :- dummy(L0, L1), f(L1, L2), dummy(L2, L).
            f([f|X], Y) :- f(X, Y).
            dummy(Xs, Xs).
        "#,
        "run.",
    );
}

#[test]
#[ignore = "Long in debug"]
fn precise_gc_4() {
    test_constant_memory(
        r#"
            run :- run(_).
            run(X) :- f(X).
            run(X) :- X == [].
            f([f|X]) :- f(X).
        "#,
        "run.",
    );
}
