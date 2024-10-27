use webpl::*;

fn unify() {
    let program: Program = vec![
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
    ];

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

    let solver = Solver::solve(&program, &query);

    for solution in solver {
        println!("{:?}", solution);
    }
}

fn main() {
    unify();
}
