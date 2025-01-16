use crate::Solver;

static APP_PROGRAM: &str = r#"
app([], L2, L2).
app([H|T], L2, [H|L3]) :- app(T, L2, L3).
"#;

#[test]
fn app() {
    let query = "app([1, 2], [3, 4], L).";
    let mut solver = Solver::new(APP_PROGRAM, query).unwrap();

    assert_eq!(solver.next(), Some(vec![("L".into(), "[1,2,3,4]".into())]));

    assert_eq!(solver.next(), None);
}

#[test]
fn recursive_solution() {
    let query = "app([1, 2], L, L).";
    let mut solver = Solver::new(APP_PROGRAM, query).unwrap();

    assert_eq!(solver.next(), Some(vec![("L".into(), "[1,2|L]".into())]));

    assert_eq!(solver.next(), None);
}

#[test]
fn mutual_recursive_solution() {
    let query_1 = "X = f(L), app([1, X], L, L).";
    let query_2 = "app([1, X], L, L), X = f(L).";
    let mut solver_1 = Solver::new(APP_PROGRAM, query_1).unwrap();
    let mut solver_2 = Solver::new(APP_PROGRAM, query_2).unwrap();

    assert_eq!(
        solver_1.next(),
        Some(vec![
            ("L".into(), "[1,f(L)|L]".into()),
            ("X".into(), "f(L)".into()),
        ])
    );

    assert_eq!(
        solver_2.next(),
        Some(vec![
            ("L".into(), "[1,f(L)|L]".into()),
            ("X".into(), "f(L)".into()),
        ])
    );

    assert_eq!(solver_1.next(), None);
    assert_eq!(solver_2.next(), None);
}

#[test]
fn backtracking() {
    let program = r#"
        generate(1).
        generate(2).
        test(2).
        solve(X) :- generate(X), test(X).
    "#;

    let query = "solve(X).";

    let mut solver = Solver::new(program, query).unwrap();

    assert_eq!(solver.next(), Some(vec![("X".into(), "2".into())]));
    assert_eq!(solver.next(), None);
}

#[test]
fn multiple_goals() {
    let program = r#"
        true.
        a :- false, true.
    "#;

    let query = "a.";

    let mut solver = Solver::new(program, query).unwrap();

    assert_eq!(solver.next(), None);
    assert_eq!(solver.next(), None);
}

#[test]
fn operator_precedence() {
    let query_1 = "X is 1 + 2 * 3.";
    let query_2 = "X is (1 + 2) * 3.";
    let query_3 = "X is 1 + 2 * 3 + 4 * 5.";

    let mut solver_1 = Solver::new("", query_1).unwrap();
    assert_eq!(solver_1.next(), Some(vec![("X".into(), "7".into())]));

    let mut solver_2 = Solver::new("", query_2).unwrap();
    assert_eq!(solver_2.next(), Some(vec![("X".into(), "9".into())]));

    let mut solver_3 = Solver::new("", query_3).unwrap();
    assert_eq!(solver_3.next(), Some(vec![("X".into(), "27".into())]));
}

#[test]
#[ignore = "long in debug"]
fn n_queens() {
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

    let solver = Solver::new(program, query).unwrap();

    assert_eq!(solver.count(), 92);
}
