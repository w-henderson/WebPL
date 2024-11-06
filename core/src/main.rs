use std::vec;

use webpl::*;

/*
take([H|T], H, T).
take([H|T], R, [H|S]) :- take(T, R, S).
perm([], []).
perm(L, [H|R]) :- take(L, H, T), perm(T,R).
generate_list(1, [1]).
generate_list(N, [N|T]) :- N > 1, M is N - 1, generate_list(M, T).
abs(0, 0).
abs(N, N) :- N > 0.
abs(N, M) :- N < 0, M is N * -1.
n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
safe_queens([]).
safe_queens([Q|Qs]) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
safe_queens([], Y, X).
safe_queens([Q|Qs], Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
*/

fn n_queens() {
    let program: Program = vec![
        // take([H|T], H, T).
        (
            CodeTerm::Compound(
                "take".into(),
                vec![
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("T".into())],
                    ),
                    CodeTerm::Var("H".into()),
                    CodeTerm::Var("T".into()),
                ],
            ),
            vec![],
        ),
        // take([H|T], R, [H|S]) :- take(T, R, S).
        (
            CodeTerm::Compound(
                "take".into(),
                vec![
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("T".into())],
                    ),
                    CodeTerm::Var("R".into()),
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("S".into())],
                    ),
                ],
            ),
            vec![CodeTerm::Compound(
                "take".into(),
                vec![
                    CodeTerm::Var("T".into()),
                    CodeTerm::Var("R".into()),
                    CodeTerm::Var("S".into()),
                ],
            )],
        ),
        // perm([], []).
        (
            CodeTerm::Compound(
                "perm".into(),
                vec![CodeTerm::Atom("[]".into()), CodeTerm::Atom("[]".into())],
            ),
            vec![],
        ),
        // perm(L, [H|R]) :- take(L, H, T), perm(T, R).
        (
            CodeTerm::Compound(
                "perm".into(),
                vec![
                    CodeTerm::Var("L".into()),
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("H".into()), CodeTerm::Var("R".into())],
                    ),
                ],
            ),
            vec![
                CodeTerm::Compound(
                    "take".into(),
                    vec![
                        CodeTerm::Var("L".into()),
                        CodeTerm::Var("H".into()),
                        CodeTerm::Var("T".into()),
                    ],
                ),
                CodeTerm::Compound(
                    "perm".into(),
                    vec![CodeTerm::Var("T".into()), CodeTerm::Var("R".into())],
                ),
            ],
        ),
        // generate_list(1, [1]).
        (
            CodeTerm::Compound(
                "generate_list".into(),
                vec![
                    CodeTerm::Atom("1".into()),
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Atom("1".into()), CodeTerm::Atom("[]".into())],
                    ),
                ],
            ),
            vec![],
        ),
        // generate_list(N, [N|T]) :- N > 1, M is N - 1, generate_list(M, T).
        (
            CodeTerm::Compound(
                "generate_list".into(),
                vec![
                    CodeTerm::Var("N".into()),
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("N".into()), CodeTerm::Var("T".into())],
                    ),
                ],
            ),
            vec![
                CodeTerm::Compound(
                    ">".into(),
                    vec![CodeTerm::Var("N".into()), CodeTerm::Atom("1".into())],
                ),
                CodeTerm::Compound(
                    "is".into(),
                    vec![
                        CodeTerm::Var("M".into()),
                        CodeTerm::Compound(
                            "-".into(),
                            vec![CodeTerm::Var("N".into()), CodeTerm::Atom("1".into())],
                        ),
                    ],
                ),
                CodeTerm::Compound(
                    "generate_list".into(),
                    vec![CodeTerm::Var("M".into()), CodeTerm::Var("T".into())],
                ),
            ],
        ),
        // abs(0, 0).
        (
            CodeTerm::Compound(
                "abs".into(),
                vec![CodeTerm::Atom("0".into()), CodeTerm::Atom("0".into())],
            ),
            vec![],
        ),
        // abs(N, N) :- N > 0.
        (
            CodeTerm::Compound(
                "abs".into(),
                vec![CodeTerm::Var("N".into()), CodeTerm::Var("N".into())],
            ),
            vec![CodeTerm::Compound(
                ">".into(),
                vec![CodeTerm::Var("N".into()), CodeTerm::Atom("0".into())],
            )],
        ),
        // abs(N, M) :- N < 0, M is N * -1.
        (
            CodeTerm::Compound(
                "abs".into(),
                vec![CodeTerm::Var("N".into()), CodeTerm::Var("M".into())],
            ),
            vec![
                CodeTerm::Compound(
                    "<".into(),
                    vec![CodeTerm::Var("N".into()), CodeTerm::Atom("0".into())],
                ),
                CodeTerm::Compound(
                    "is".into(),
                    vec![
                        CodeTerm::Var("M".into()),
                        CodeTerm::Compound(
                            "*".into(),
                            vec![CodeTerm::Var("N".into()), CodeTerm::Atom("-1".into())],
                        ),
                    ],
                ),
            ],
        ),
        // n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
        (
            CodeTerm::Compound(
                "n_queens".into(),
                vec![CodeTerm::Var("N".into()), CodeTerm::Var("Qs".into())],
            ),
            vec![
                CodeTerm::Compound(
                    "generate_list".into(),
                    vec![CodeTerm::Var("N".into()), CodeTerm::Var("Qs1".into())],
                ),
                CodeTerm::Compound(
                    "perm".into(),
                    vec![CodeTerm::Var("Qs1".into()), CodeTerm::Var("Qs".into())],
                ),
                CodeTerm::Compound("safe_queens".into(), vec![CodeTerm::Var("Qs".into())]),
            ],
        ),
        // safe_queens([]).
        (
            CodeTerm::Compound("safe_queens".into(), vec![CodeTerm::Atom("[]".into())]),
            vec![],
        ),
        // safe_queens([Q|Qs]) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
        (
            CodeTerm::Compound(
                "safe_queens".into(),
                vec![CodeTerm::Compound(
                    ".".into(),
                    vec![CodeTerm::Var("Q".into()), CodeTerm::Var("Qs".into())],
                )],
            ),
            vec![
                CodeTerm::Compound(
                    "safe_queens".into(),
                    vec![
                        CodeTerm::Var("Qs".into()),
                        CodeTerm::Var("Q".into()),
                        CodeTerm::Atom("1".into()),
                    ],
                ),
                CodeTerm::Compound("safe_queens".into(), vec![CodeTerm::Var("Qs".into())]),
            ],
        ),
        // safe_queens([], Y, X).
        (
            CodeTerm::Compound(
                "safe_queens".into(),
                vec![
                    CodeTerm::Atom("[]".into()),
                    CodeTerm::Var("Y".into()),
                    CodeTerm::Var("X".into()),
                ],
            ),
            vec![],
        ),
        // safe_queens([Q|Qs], Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
        (
            CodeTerm::Compound(
                "safe_queens".into(),
                vec![
                    CodeTerm::Compound(
                        ".".into(),
                        vec![CodeTerm::Var("Q".into()), CodeTerm::Var("Qs".into())],
                    ),
                    CodeTerm::Var("Q0".into()),
                    CodeTerm::Var("D0".into()),
                ],
            ),
            vec![
                CodeTerm::Compound(
                    "=\\=".into(),
                    vec![CodeTerm::Var("Q0".into()), CodeTerm::Var("Q".into())],
                ),
                CodeTerm::Compound(
                    "is".into(),
                    vec![
                        CodeTerm::Var("Diff".into()),
                        CodeTerm::Compound(
                            "-".into(),
                            vec![CodeTerm::Var("Q0".into()), CodeTerm::Var("Q".into())],
                        ),
                    ],
                ),
                CodeTerm::Compound(
                    "abs".into(),
                    vec![
                        CodeTerm::Var("Diff".into()),
                        CodeTerm::Var("AbsDiff".into()),
                    ],
                ),
                CodeTerm::Compound(
                    "=\\=".into(),
                    vec![CodeTerm::Var("AbsDiff".into()), CodeTerm::Var("D0".into())],
                ),
                CodeTerm::Compound(
                    "is".into(),
                    vec![
                        CodeTerm::Var("D1".into()),
                        CodeTerm::Compound(
                            "+".into(),
                            vec![CodeTerm::Var("D0".into()), CodeTerm::Atom("1".into())],
                        ),
                    ],
                ),
                CodeTerm::Compound(
                    "safe_queens".into(),
                    vec![
                        CodeTerm::Var("Qs".into()),
                        CodeTerm::Var("Q0".into()),
                        CodeTerm::Var("D1".into()),
                    ],
                ),
            ],
        ),
    ];

    let query: Query = vec![CodeTerm::Compound(
        "n_queens".into(),
        vec![CodeTerm::Atom("8".into()), CodeTerm::Var("Qs".into())],
    )];

    let solver = Solver::solve(&program, &query);

    for solution in solver {
        println!("{:?}", solution);
    }
}

fn main() {
    n_queens();
}
