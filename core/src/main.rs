use webpl::*;

fn unify() {
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

    let solver = Solver::solve(&program, &query);

    for solution in solver {
        println!("{:?}", solution);
    }
}

fn main() {
    unify();
}
