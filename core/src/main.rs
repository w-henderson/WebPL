use std::vec;

use webpl::ast::*;
use webpl::Solver;

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
            ASTTerm::Compound(
                "take".into(),
                vec![
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("T".into())],
                    ),
                    ASTTerm::Var("H".into()),
                    ASTTerm::Var("T".into()),
                ],
            ),
            vec![],
        ),
        // take([H|T], R, [H|S]) :- take(T, R, S).
        (
            ASTTerm::Compound(
                "take".into(),
                vec![
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("T".into())],
                    ),
                    ASTTerm::Var("R".into()),
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("S".into())],
                    ),
                ],
            ),
            vec![ASTTerm::Compound(
                "take".into(),
                vec![
                    ASTTerm::Var("T".into()),
                    ASTTerm::Var("R".into()),
                    ASTTerm::Var("S".into()),
                ],
            )],
        ),
        // perm([], []).
        (
            ASTTerm::Compound(
                "perm".into(),
                vec![ASTTerm::Atom("[]".into()), ASTTerm::Atom("[]".into())],
            ),
            vec![],
        ),
        // perm(L, [H|R]) :- take(L, H, T), perm(T, R).
        (
            ASTTerm::Compound(
                "perm".into(),
                vec![
                    ASTTerm::Var("L".into()),
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("H".into()), ASTTerm::Var("R".into())],
                    ),
                ],
            ),
            vec![
                ASTTerm::Compound(
                    "take".into(),
                    vec![
                        ASTTerm::Var("L".into()),
                        ASTTerm::Var("H".into()),
                        ASTTerm::Var("T".into()),
                    ],
                ),
                ASTTerm::Compound(
                    "perm".into(),
                    vec![ASTTerm::Var("T".into()), ASTTerm::Var("R".into())],
                ),
            ],
        ),
        // generate_list(1, [1]).
        (
            ASTTerm::Compound(
                "generate_list".into(),
                vec![
                    ASTTerm::Atom("1".into()),
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Atom("1".into()), ASTTerm::Atom("[]".into())],
                    ),
                ],
            ),
            vec![],
        ),
        // generate_list(N, [N|T]) :- N > 1, M is N - 1, generate_list(M, T).
        (
            ASTTerm::Compound(
                "generate_list".into(),
                vec![
                    ASTTerm::Var("N".into()),
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("N".into()), ASTTerm::Var("T".into())],
                    ),
                ],
            ),
            vec![
                ASTTerm::Compound(
                    ">".into(),
                    vec![ASTTerm::Var("N".into()), ASTTerm::Atom("1".into())],
                ),
                ASTTerm::Compound(
                    "is".into(),
                    vec![
                        ASTTerm::Var("M".into()),
                        ASTTerm::Compound(
                            "-".into(),
                            vec![ASTTerm::Var("N".into()), ASTTerm::Atom("1".into())],
                        ),
                    ],
                ),
                ASTTerm::Compound(
                    "generate_list".into(),
                    vec![ASTTerm::Var("M".into()), ASTTerm::Var("T".into())],
                ),
            ],
        ),
        // abs(0, 0).
        (
            ASTTerm::Compound(
                "abs".into(),
                vec![ASTTerm::Atom("0".into()), ASTTerm::Atom("0".into())],
            ),
            vec![],
        ),
        // abs(N, N) :- N > 0.
        (
            ASTTerm::Compound(
                "abs".into(),
                vec![ASTTerm::Var("N".into()), ASTTerm::Var("N".into())],
            ),
            vec![ASTTerm::Compound(
                ">".into(),
                vec![ASTTerm::Var("N".into()), ASTTerm::Atom("0".into())],
            )],
        ),
        // abs(N, M) :- N < 0, M is N * -1.
        (
            ASTTerm::Compound(
                "abs".into(),
                vec![ASTTerm::Var("N".into()), ASTTerm::Var("M".into())],
            ),
            vec![
                ASTTerm::Compound(
                    "<".into(),
                    vec![ASTTerm::Var("N".into()), ASTTerm::Atom("0".into())],
                ),
                ASTTerm::Compound(
                    "is".into(),
                    vec![
                        ASTTerm::Var("M".into()),
                        ASTTerm::Compound(
                            "*".into(),
                            vec![ASTTerm::Var("N".into()), ASTTerm::Atom("-1".into())],
                        ),
                    ],
                ),
            ],
        ),
        // n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
        (
            ASTTerm::Compound(
                "n_queens".into(),
                vec![ASTTerm::Var("N".into()), ASTTerm::Var("Qs".into())],
            ),
            vec![
                ASTTerm::Compound(
                    "generate_list".into(),
                    vec![ASTTerm::Var("N".into()), ASTTerm::Var("Qs1".into())],
                ),
                ASTTerm::Compound(
                    "perm".into(),
                    vec![ASTTerm::Var("Qs1".into()), ASTTerm::Var("Qs".into())],
                ),
                ASTTerm::Compound("safe_queens".into(), vec![ASTTerm::Var("Qs".into())]),
            ],
        ),
        // safe_queens([]).
        (
            ASTTerm::Compound("safe_queens".into(), vec![ASTTerm::Atom("[]".into())]),
            vec![],
        ),
        // safe_queens([Q|Qs]) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
        (
            ASTTerm::Compound(
                "safe_queens".into(),
                vec![ASTTerm::Compound(
                    ".".into(),
                    vec![ASTTerm::Var("Q".into()), ASTTerm::Var("Qs".into())],
                )],
            ),
            vec![
                ASTTerm::Compound(
                    "safe_queens".into(),
                    vec![
                        ASTTerm::Var("Qs".into()),
                        ASTTerm::Var("Q".into()),
                        ASTTerm::Atom("1".into()),
                    ],
                ),
                ASTTerm::Compound("safe_queens".into(), vec![ASTTerm::Var("Qs".into())]),
            ],
        ),
        // safe_queens([], Y, X).
        (
            ASTTerm::Compound(
                "safe_queens".into(),
                vec![
                    ASTTerm::Atom("[]".into()),
                    ASTTerm::Var("Y".into()),
                    ASTTerm::Var("X".into()),
                ],
            ),
            vec![],
        ),
        // safe_queens([Q|Qs], Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
        (
            ASTTerm::Compound(
                "safe_queens".into(),
                vec![
                    ASTTerm::Compound(
                        ".".into(),
                        vec![ASTTerm::Var("Q".into()), ASTTerm::Var("Qs".into())],
                    ),
                    ASTTerm::Var("Q0".into()),
                    ASTTerm::Var("D0".into()),
                ],
            ),
            vec![
                ASTTerm::Compound(
                    "=\\=".into(),
                    vec![ASTTerm::Var("Q0".into()), ASTTerm::Var("Q".into())],
                ),
                ASTTerm::Compound(
                    "is".into(),
                    vec![
                        ASTTerm::Var("Diff".into()),
                        ASTTerm::Compound(
                            "-".into(),
                            vec![ASTTerm::Var("Q0".into()), ASTTerm::Var("Q".into())],
                        ),
                    ],
                ),
                ASTTerm::Compound(
                    "abs".into(),
                    vec![ASTTerm::Var("Diff".into()), ASTTerm::Var("AbsDiff".into())],
                ),
                ASTTerm::Compound(
                    "=\\=".into(),
                    vec![ASTTerm::Var("AbsDiff".into()), ASTTerm::Var("D0".into())],
                ),
                ASTTerm::Compound(
                    "is".into(),
                    vec![
                        ASTTerm::Var("D1".into()),
                        ASTTerm::Compound(
                            "+".into(),
                            vec![ASTTerm::Var("D0".into()), ASTTerm::Atom("1".into())],
                        ),
                    ],
                ),
                ASTTerm::Compound(
                    "safe_queens".into(),
                    vec![
                        ASTTerm::Var("Qs".into()),
                        ASTTerm::Var("Q0".into()),
                        ASTTerm::Var("D1".into()),
                    ],
                ),
            ],
        ),
    ];

    let query: Query = vec![ASTTerm::Compound(
        "n_queens".into(),
        vec![ASTTerm::Atom("8".into()), ASTTerm::Var("Qs".into())],
    )];

    let (program, query, string_map) = to_code_term(program, query);

    let solver = Solver::solve(&program, string_map, &query);

    for solution in solver {
        println!("{:?}", solution);
    }
}

fn main() {
    n_queens();
}
