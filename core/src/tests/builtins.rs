use crate::Solver;

#[test]
fn cut() {
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

    let mut solver_1 = Solver::new(program, query_1).unwrap();
    assert_eq!(solver_1.next(), Some(vec![("X".into(), "2".into())]));
    assert_eq!(solver_1.next(), None);

    let mut solver_2 = Solver::new(program, query_2).unwrap();
    assert_eq!(solver_2.next(), Some(vec![("X".into(), "1".into())]));
    assert_eq!(solver_2.next(), None);

    let mut solver_3 = Solver::new(program, query_3).unwrap();
    assert_eq!(solver_3.next(), Some(vec![]));
    assert_eq!(solver_3.next(), None);
}

#[test]
fn is() {
    let query = "Y = 3, X is Y + (2 * 5.1).";

    let mut solver = Solver::new("", query).unwrap();

    assert_eq!(
        solver.next(),
        Some(vec![("Y".into(), "3".into()), ("X".into(), "13.2".into())])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn cmp() {
    let query_1 = "4 > 3.";
    let query_2 = "3 > 4.";

    let mut solver_1 = Solver::new("", query_1).unwrap();
    assert_eq!(solver_1.next(), Some(vec![]));
    assert_eq!(solver_1.next(), None);

    let mut solver_2 = Solver::new("", query_2).unwrap();
    assert_eq!(solver_2.next(), None);
}
