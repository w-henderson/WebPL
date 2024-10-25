use webpl::*;

fn unify() {
    let program: Program = vec![
        (
            CodeTerm::Compound("a".into(), vec![CodeTerm::Var("X".into())]),
            vec![CodeTerm::Compound(
                "b".into(),
                vec![CodeTerm::Var("X".into())],
            )],
        ),
        (
            CodeTerm::Compound("b".into(), vec![CodeTerm::Var("X".into())]),
            vec![],
        ),
    ];

    let query: Query = vec![CodeTerm::Compound(
        "a".into(),
        vec![CodeTerm::Var("X".into())],
    )];

    let solver = Solver::solve(&program, &query);

    for solution in solver {
        println!("{:?}", solution);
    }
}

fn main() {
    unify();
}
