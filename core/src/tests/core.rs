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
