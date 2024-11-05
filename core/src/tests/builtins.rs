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
