use crate::ast::*;
use crate::Solver;

#[test]
fn is() {
    let program: Program = Program::default();

    let query: Query = vec![
        ASTTerm::Compound(
            "=".into(),
            vec![ASTTerm::Var("Y".into()), ASTTerm::Atom("3".into())],
        ),
        ASTTerm::Compound(
            "is".into(),
            vec![
                ASTTerm::Var("X".into()),
                ASTTerm::Compound(
                    "+".into(),
                    vec![
                        ASTTerm::Var("Y".into()),
                        ASTTerm::Compound(
                            "*".into(),
                            vec![ASTTerm::Atom("2".into()), ASTTerm::Atom("5.1".into())],
                        ),
                    ],
                ),
            ],
        ),
    ];

    let (program, query, string_map) = to_code_term(program, query);

    let mut solver = Solver::solve(&program, string_map, &query);

    assert_eq!(
        solver.next(),
        Some(vec![("Y".into(), "3".into()), ("X".into(), "13.2".into())])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn cmp() {
    let query_1: Query = vec![ASTTerm::Compound(
        ">".into(),
        vec![ASTTerm::Atom("4".into()), ASTTerm::Atom("3".into())],
    )];
    let query_2: Query = vec![ASTTerm::Compound(
        ">".into(),
        vec![ASTTerm::Atom("3".into()), ASTTerm::Atom("4".into())],
    )];

    let (program_1, query_1, string_map_1) = to_code_term(Program::default(), query_1);
    let (program_2, query_2, string_map_2) = to_code_term(Program::default(), query_2);

    let mut solver_1 = Solver::solve(&program_1, string_map_1, &query_1);
    let mut solver_2 = Solver::solve(&program_2, string_map_2, &query_2);

    assert_eq!(solver_1.next(), Some(vec![]));
    assert_eq!(solver_1.next(), None);
    assert_eq!(solver_2.next(), None);
}
