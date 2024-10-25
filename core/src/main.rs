use webpl::*;

fn unify() {
    let program: Program = vec![
        (
            CodeTerm::Compound((
                "college".into(),
                vec![
                    CodeTerm::Atom("william".into()),
                    CodeTerm::Atom("churchill".into()),
                ],
            )),
            vec![],
        ),
        (
            CodeTerm::Compound((
                "college".into(),
                vec![
                    CodeTerm::Atom("john".into()),
                    CodeTerm::Atom("churchill".into()),
                ],
            )),
            vec![],
        ),
        (
            CodeTerm::Compound((
                "college".into(),
                vec![
                    CodeTerm::Atom("elizabeth".into()),
                    CodeTerm::Atom("trinity".into()),
                ],
            )),
            vec![],
        ),
    ];

    let query: Query = vec![CodeTerm::Compound((
        "college".into(),
        vec![CodeTerm::Var(0), CodeTerm::Atom("churchill".into())],
    ))];

    solve(program, query);
}

fn main() {
    unify();
}
