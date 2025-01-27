use crate::tests::SolverFn;
use crate::{test, Solver};

static APP_PROGRAM: &str = r#"
app([], L2, L2).
app([H|T], L2, [H|L3]) :- app(T, L2, L3).
"#;

test!(app, |solver: SolverFn| {
    let query = "app([1, 2], [3, 4], L).";
    let mut solver = solver(APP_PROGRAM, query);

    assert_eq!(
        solver.step().unwrap(),
        Some(vec![("L".into(), "[1,2,3,4]".into())])
    );

    assert_eq!(solver.step().unwrap(), None);
});

test!(recursive_solution, |solver: SolverFn| {
    let query = "app([1, 2], L, L).";
    let mut solver = solver(APP_PROGRAM, query);

    assert_eq!(
        solver.step().unwrap(),
        Some(vec![("L".into(), "[1,2|L]".into())])
    );

    assert_eq!(solver.step().unwrap(), None);
});

test!(mutual_recursive_solution, |solver: SolverFn| {
    let query_1 = "X = f(L), app([1, X], L, L).";
    let query_2 = "app([1, X], L, L), X = f(L).";
    let mut solver_1 = solver(APP_PROGRAM, query_1);
    let mut solver_2 = solver(APP_PROGRAM, query_2);

    assert_eq!(
        solver_1.step().unwrap(),
        Some(vec![
            ("L".into(), "[1,X|L]".into()),
            ("X".into(), "f([1,X|L])".into()),
        ])
    );

    assert_eq!(
        solver_2.step().unwrap(),
        Some(vec![
            ("L".into(), "[1,X|L]".into()),
            ("X".into(), "f(L)".into()),
        ])
    );

    assert_eq!(solver_1.step().unwrap(), None);
    assert_eq!(solver_2.step().unwrap(), None);
});

test!(backtracking, |solver: SolverFn| {
    let program = r#"
        generate(1).
        generate(2).
        test(2).
        solve(X) :- generate(X), test(X).
    "#;

    let query = "solve(X).";

    let mut solver = solver(program, query);

    assert_eq!(solver.step().unwrap(), Some(vec![("X".into(), "2".into())]));
    assert_eq!(solver.step().unwrap(), None);
});

test!(multiple_goals, |solver: SolverFn| {
    let program = r#"
        true.
        a :- false, true.
    "#;

    let query = "a.";

    let mut solver = solver(program, query);

    assert_eq!(solver.step().unwrap(), None);
    assert_eq!(solver.step().unwrap(), None);
});

test!(operator_precedence, |solver: SolverFn| {
    let query_1 = "X is 1 + 2 * 3.";
    let query_2 = "X is (1 + 2) * 3.";
    let query_3 = "X is 1 + 2 * 3 + 4 * 5.";

    let mut solver_1 = solver("", query_1);
    assert_eq!(
        solver_1.step().unwrap(),
        Some(vec![("X".into(), "7".into())])
    );

    let mut solver_2 = solver("", query_2);
    assert_eq!(
        solver_2.step().unwrap(),
        Some(vec![("X".into(), "9".into())])
    );

    let mut solver_3 = solver("", query_3);
    assert_eq!(
        solver_3.step().unwrap(),
        Some(vec![("X".into(), "27".into())])
    );
});

test!(LONG n_queens, |solver: SolverFn| {
    let program = r#"
        take([H|T], H, T).
        take([H|T], R, [H|S]) :- take(T, R, S).
        perm([], []).
        perm(L, [H|R]) :- take(L, H, T), perm(T,R).
        generate_list(1, [1]).
        generate_list(N, [N|T]) :- N > 1, M is N - 1, generate_list(M, T).
        abs(0, 0).
        abs(N, N) :- N > 0.
        abs(N, M) :- N < 0, M is N * -1.
        n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
        safe_queens([]).
        safe_queens([Q|Qs]) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
        safe_queens([], Y, X).
        safe_queens([Q|Qs], Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
    "#;

    let query = "n_queens(8, Qs).";

    let solver = solver(program, query);

    assert_eq!(solver.count(), 92);
});

#[test]
fn empty() {
    assert!(Solver::new("", "a.").is_ok());
    assert!(Solver::new("a.", "").is_ok());
    assert!(Solver::new("", "").is_ok());
}
