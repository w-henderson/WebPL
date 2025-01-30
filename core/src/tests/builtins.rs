use crate::tests::SolverFn;
use crate::{test, Error, Solver};

test!(cut, |solver: SolverFn| {
    let program = r#"
        a(1).
        a(2).
        a(3).

        b(2).
        b(3).

        c(2).
        c(3).

        d(X) :- a(X), b(X), !, c(X).

        e(1) :- !.
        e(2).
    "#;

    let query_1 = "d(X).";
    let query_2 = "e(X).";
    let query_3 = "e(2).";

    let mut solver_1 = solver(program, query_1);
    assert_eq!(
        solver_1.step().unwrap(),
        Some(vec![("X".into(), "2".into())])
    );
    assert_eq!(solver_1.step().unwrap(), None);

    let mut solver_2 = solver(program, query_2);
    assert_eq!(
        solver_2.step().unwrap(),
        Some(vec![("X".into(), "1".into())])
    );
    assert_eq!(solver_2.step().unwrap(), None);

    let mut solver_3 = solver(program, query_3);
    assert_eq!(solver_3.step().unwrap(), Some(vec![]));
    assert_eq!(solver_3.step().unwrap(), None);
});

test!(cut_failure, |solver: SolverFn| {
    let program = r#"
        a :- !, 1 =:= 2.
        a.
        b :- a.
        b.
    "#;

    let query_1 = "a.";
    let query_2 = "b.";

    let mut solver_1 = solver(program, query_1);
    assert_eq!(solver_1.step().unwrap(), None);

    let mut solver_2 = solver(program, query_2);
    assert_eq!(solver_2.step().unwrap(), Some(vec![]));
    assert_eq!(solver_2.step().unwrap(), None);
});

test!(is, |solver: SolverFn| {
    let query = "Y = 3, X is Y + (2 * 5.1).";

    let mut solver = solver("", query);

    assert_eq!(
        solver.step().unwrap(),
        Some(vec![("Y".into(), "3".into()), ("X".into(), "13.2".into())])
    );

    assert_eq!(solver.step().unwrap(), None);
});

test!(intdiv, |solver: SolverFn| {
    let mut solver_1 = solver("", "X is 10 // 3.");
    assert_eq!(
        solver_1.step().unwrap(),
        Some(vec![("X".into(), "3".into())])
    );
    assert_eq!(solver_1.step().unwrap(), None);

    let mut solver_2 = solver("", "X is 10 mod 3.");
    assert_eq!(
        solver_2.step().unwrap(),
        Some(vec![("X".into(), "1".into())])
    );
    assert_eq!(solver_2.step().unwrap(), None);
});

test!(cmp, |solver: SolverFn| {
    let query_1 = "4 > 3.";
    let query_2 = "3 > 4.";
    let query_3 = "5 > 2 + 2.";

    let mut solver_1 = solver("", query_1);
    assert_eq!(solver_1.step().unwrap(), Some(vec![]));
    assert_eq!(solver_1.step().unwrap(), None);

    let mut solver_2 = solver("", query_2);
    assert_eq!(solver_2.step().unwrap(), None);

    let mut solver_3 = solver("", query_3);
    assert_eq!(solver_3.step().unwrap(), Some(vec![]));
    assert_eq!(solver_3.step().unwrap(), None);
});

test!(insufficient_instantiation, |solver: SolverFn| {
    let mut solver = solver("", "X is Y.");
    if let Err(e) = solver.step() {
        assert_eq!(
            e,
            Error {
                location: None,
                error: "Insufficiently instantiated variable `_1`".into()
            }
        );
    } else {
        panic!("Expected an error");
    }
});
