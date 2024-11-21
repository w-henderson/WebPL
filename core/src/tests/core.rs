use crate::*;

fn app_program() -> Program {
    vec![
        (
            CodeTerm::Compound(
                "app".into(),
                vec![
                    CodeTerm::Atom("nil".into()),
                    CodeTerm::Var("L2".into()),
                    CodeTerm::Var("L2".into()),
                ],
            ),
            vec![],
        ),
        (
            CodeTerm::Compound(
                "app".into(),
                vec![
                    CodeTerm::Compound(
                        "cons".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("T".into())],
                    ),
                    CodeTerm::Var("L2".into()),
                    CodeTerm::Compound(
                        "cons".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("L3".into())],
                    ),
                ],
            ),
            vec![CodeTerm::Compound(
                "app".into(),
                vec![
                    CodeTerm::Var("T".into()),
                    CodeTerm::Var("L2".into()),
                    CodeTerm::Var("L3".into()),
                ],
            )],
        ),
    ]
}

#[test]
fn app() {
    let program: Program = app_program();

    let query: Query = vec![CodeTerm::Compound(
        "app".into(),
        vec![
            CodeTerm::Compound(
                "cons".into(),
                vec![
                    CodeTerm::Atom("1".into()),
                    CodeTerm::Compound(
                        "cons".into(),
                        vec![CodeTerm::Atom("2".into()), CodeTerm::Atom("nil".into())],
                    ),
                ],
            ),
            CodeTerm::Compound(
                "cons".into(),
                vec![
                    CodeTerm::Atom("3".into()),
                    CodeTerm::Compound(
                        "cons".into(),
                        vec![CodeTerm::Atom("4".into()), CodeTerm::Atom("nil".into())],
                    ),
                ],
            ),
            CodeTerm::Var("L".into()),
        ],
    )];

    let mut solver = Solver::solve(&program, &query);

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
    let program: Program = app_program();

    let query: Query = vec![CodeTerm::Compound(
        "app".into(),
        vec![
            CodeTerm::Compound(
                "cons".into(),
                vec![
                    CodeTerm::Atom("1".into()),
                    CodeTerm::Compound(
                        "cons".into(),
                        vec![CodeTerm::Atom("2".into()), CodeTerm::Atom("nil".into())],
                    ),
                ],
            ),
            CodeTerm::Var("L".into()),
            CodeTerm::Var("L".into()),
        ],
    )];

    let mut solver = Solver::solve(&program, &query);

    assert_eq!(
        solver.next(),
        Some(vec![("L".into(), "cons(1, cons(2, L))".into())])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn backtracking() {
    let program: Program = vec![
        (
            CodeTerm::Compound("generate".into(), vec![CodeTerm::Atom("1".into())]),
            vec![],
        ),
        (
            CodeTerm::Compound("generate".into(), vec![CodeTerm::Atom("2".into())]),
            vec![],
        ),
        (
            CodeTerm::Compound("test".into(), vec![CodeTerm::Atom("2".into())]),
            vec![],
        ),
        (
            CodeTerm::Compound("solve".into(), vec![CodeTerm::Var("X".into())]),
            vec![
                CodeTerm::Compound("generate".into(), vec![CodeTerm::Var("X".into())]),
                CodeTerm::Compound("test".into(), vec![CodeTerm::Var("X".into())]),
            ],
        ),
    ];

    let query = vec![CodeTerm::Compound(
        "solve".into(),
        vec![CodeTerm::Var("X".into())],
    )];

    let mut solver = Solver::solve(&program, &query);

    assert_eq!(solver.next(), Some(vec![("X".into(), "2".into())]));

    assert_eq!(solver.next(), None);
}

#[test]
fn multiple_goals() {
    let program: Program = vec![
        (CodeTerm::Atom("true".into()), vec![]),
        (
            CodeTerm::Atom("a".into()),
            vec![
                CodeTerm::Atom("false".into()),
                CodeTerm::Atom("true".into()),
            ],
        ),
    ];

    let query = vec![CodeTerm::Atom("a".into())];

    let mut solver = Solver::solve(&program, &query);
    assert_eq!(solver.next(), None);
    assert_eq!(solver.next(), None);
}
