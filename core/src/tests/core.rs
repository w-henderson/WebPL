use crate::ast::*;
use crate::Solver;

fn app_program() -> Program {
    vec![
        (
            ASTTerm::Compound(
                "app".into(),
                vec![
                    ASTTerm::Atom("nil".into()),
                    ASTTerm::Var("L2".into()),
                    ASTTerm::Var("L2".into()),
                ],
            ),
            vec![],
        ),
        (
            ASTTerm::Compound(
                "app".into(),
                vec![
                    ASTTerm::Compound(
                        "cons".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("T".into())],
                    ),
                    ASTTerm::Var("L2".into()),
                    ASTTerm::Compound(
                        "cons".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("L3".into())],
                    ),
                ],
            ),
            vec![ASTTerm::Compound(
                "app".into(),
                vec![
                    ASTTerm::Var("T".into()),
                    ASTTerm::Var("L2".into()),
                    ASTTerm::Var("L3".into()),
                ],
            )],
        ),
    ]
}

#[test]
fn app() {
    let program: Program = app_program();

    let query: Query = vec![ASTTerm::Compound(
        "app".into(),
        vec![
            ASTTerm::Compound(
                "cons".into(),
                vec![
                    ASTTerm::Atom("1".into()),
                    ASTTerm::Compound(
                        "cons".into(),
                        vec![ASTTerm::Atom("2".into()), ASTTerm::Atom("nil".into())],
                    ),
                ],
            ),
            ASTTerm::Compound(
                "cons".into(),
                vec![
                    ASTTerm::Atom("3".into()),
                    ASTTerm::Compound(
                        "cons".into(),
                        vec![ASTTerm::Atom("4".into()), ASTTerm::Atom("nil".into())],
                    ),
                ],
            ),
            ASTTerm::Var("L".into()),
        ],
    )];

    let (program, query, string_map) = to_code_term(program, query);

    let mut solver = Solver::solve(&program, string_map, &query);

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

    let query: Query = vec![ASTTerm::Compound(
        "app".into(),
        vec![
            ASTTerm::Compound(
                "cons".into(),
                vec![
                    ASTTerm::Atom("1".into()),
                    ASTTerm::Compound(
                        "cons".into(),
                        vec![ASTTerm::Atom("2".into()), ASTTerm::Atom("nil".into())],
                    ),
                ],
            ),
            ASTTerm::Var("L".into()),
            ASTTerm::Var("L".into()),
        ],
    )];

    let (program, query, string_map) = to_code_term(program, query);

    let mut solver = Solver::solve(&program, string_map, &query);

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
            ASTTerm::Compound("generate".into(), vec![ASTTerm::Atom("1".into())]),
            vec![],
        ),
        (
            ASTTerm::Compound("generate".into(), vec![ASTTerm::Atom("2".into())]),
            vec![],
        ),
        (
            ASTTerm::Compound("test".into(), vec![ASTTerm::Atom("2".into())]),
            vec![],
        ),
        (
            ASTTerm::Compound("solve".into(), vec![ASTTerm::Var("X".into())]),
            vec![
                ASTTerm::Compound("generate".into(), vec![ASTTerm::Var("X".into())]),
                ASTTerm::Compound("test".into(), vec![ASTTerm::Var("X".into())]),
            ],
        ),
    ];

    let query = vec![ASTTerm::Compound(
        "solve".into(),
        vec![ASTTerm::Var("X".into())],
    )];

    let (program, query, string_map) = to_code_term(program, query);

    let mut solver = Solver::solve(&program, string_map, &query);

    assert_eq!(solver.next(), Some(vec![("X".into(), "2".into())]));

    assert_eq!(solver.next(), None);
}

#[test]
fn multiple_goals() {
    let program: Program = vec![
        (ASTTerm::Atom("true".into()), vec![]),
        (
            ASTTerm::Atom("a".into()),
            vec![ASTTerm::Atom("false".into()), ASTTerm::Atom("true".into())],
        ),
    ];

    let query = vec![ASTTerm::Atom("a".into())];

    let (program, query, string_map) = to_code_term(program, query);

    let mut solver = Solver::solve(&program, string_map, &query);

    assert_eq!(solver.next(), None);
    assert_eq!(solver.next(), None);
}
