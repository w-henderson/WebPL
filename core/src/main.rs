use webpl::WebPL;

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
safe_queens([Q|Qs], Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).*/

static PROGRAM: &str = r#"
take(cons(H, T), H, T).
take(cons(H, T), R, cons(H, S)) :- take(T, R, S).
perm(nil, nil).
perm(L, cons(H, R)) :- take(L, H, T), perm(T,R).
generate_list(1, cons(1, nil)).
generate_list(N, cons(N, T)) :- N > 1, M is N - 1, generate_list(M, T).
abs(0, 0).
abs(N, N) :- N > 0.
abs(N, M) :- N < 0, M is N * -1.
n_queens(N, Qs) :- generate_list(N, Qs1), perm(Qs1, Qs), safe_queens(Qs).
safe_queens(nil).
safe_queens(cons(Q, Qs)) :- safe_queens(Qs, Q, 1), safe_queens(Qs).
safe_queens(nil, Y, X).
safe_queens(cons(Q, Qs), Q0, D0) :- Q0 =\= Q, Diff is Q0 - Q, abs(Diff, AbsDiff), AbsDiff =\= D0, D1 is D0 + 1, safe_queens(Qs, Q0, D1).
"#;

static QUERY: &str = r#"n_queens(8, Qs)."#;

fn main() {
    let mut webpl = WebPL::new(PROGRAM).unwrap();
    let solver = webpl.solve(QUERY).unwrap();

    for solution in solver {
        println!("{:?}", solution);
    }
}
