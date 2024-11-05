use crate::*;

#[test]
fn is() {
    let program: Program = Program::default();

    let query: Query = vec![
        CodeTerm::Compound(
            "=".into(),
            vec![CodeTerm::Var("Y".into()), CodeTerm::Atom("3".into())],
        ),
        CodeTerm::Compound(
            "is".into(),
            vec![
                CodeTerm::Var("X".into()),
                CodeTerm::Compound(
                    "+".into(),
                    vec![
                        CodeTerm::Var("Y".into()),
                        CodeTerm::Compound(
                            "*".into(),
                            vec![CodeTerm::Atom("2".into()), CodeTerm::Atom("5.1".into())],
                        ),
                    ],
                ),
            ],
        ),
    ];

    let mut solver = Solver::solve(&program, &query);

    assert_eq!(
        solver.next(),
        Some(vec![("Y".into(), "3".into()), ("X".into(), "13.2".into())])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn cmp() {
    let program: Program = Program::default();
    let query_1: Query = vec![CodeTerm::Compound(
        ">".into(),
        vec![CodeTerm::Atom("4".into()), CodeTerm::Atom("3".into())],
    )];
    let query_2: Query = vec![CodeTerm::Compound(
        ">".into(),
        vec![CodeTerm::Atom("3".into()), CodeTerm::Atom("4".into())],
    )];
    let mut solver_1 = Solver::solve(&program, &query_1);
    let mut solver_2 = Solver::solve(&program, &query_2);

    assert_eq!(solver_1.next(), Some(vec![]));
    assert_eq!(solver_1.next(), None);
    assert_eq!(solver_2.next(), None);
}
