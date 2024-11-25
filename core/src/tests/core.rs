use crate::WebPL;

static APP_PROGRAM: &str = r#"
app(nil, L2, L2).
app(cons(H, T), L2, cons(H, L3)) :- app(T, L2, L3).
"#;

#[test]
fn app() {
    let query = "app(cons(1, cons(2, nil)), cons(3, cons(4, nil)), L).";
    let mut webpl = WebPL::new(APP_PROGRAM).unwrap();
    let mut solver = webpl.solve(query).unwrap();

    assert_eq!(
        solver.next(),
        Some(vec![(
            "L".into(),
            "cons(1, cons(2, cons(3, cons(4, nil))))".into()
        )])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn recursive_solution() {
    let query = "app(cons(1, cons(2, nil)), L, L).";
    let mut webpl = WebPL::new(APP_PROGRAM).unwrap();
    let mut solver = webpl.solve(query).unwrap();

    assert_eq!(
        solver.next(),
        Some(vec![("L".into(), "cons(1, cons(2, L))".into())])
    );

    assert_eq!(solver.next(), None);
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
    let mut webpl = WebPL::new(program).unwrap();
    let mut solver = webpl.solve(query).unwrap();

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

    let mut webpl = WebPL::new(program).unwrap();
    let mut solver = webpl.solve(query).unwrap();

    assert_eq!(solver.next(), None);
    assert_eq!(solver.next(), None);
}

#[test]
#[ignore = "long in debug"]
fn n_queens() {
    let program = r#"
        take(cons(H, T), H, T).
        take(cons(H, T), R, cons(H, S)) :- take(T, R, S).
        perm(nil, nil).
        perm(L, cons(H, R)) :- take(L, H, T), perm(T,R).
        generate_list(1, cons(1, nil)).
        generate_list(N, cons(N, T)) :- N > 1, M is N - 1, generate_list(M, T).
        abs(0, 0).
        abs(N, N) :- N > 0.
        abs(N, M) :- N < 0, M is N * -1.
        n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
        safe_queens(nil).
        safe_queens(cons(Q, Qs)) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
        safe_queens(nil, Y, X).
        safe_queens(cons(Q, Qs), Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
    "#;

    let query = "n_queens(8, Qs).";

    let mut webpl = WebPL::new(program).unwrap();
    let solver = webpl.solve(query).unwrap();

    assert_eq!(solver.count(), 92);
}
