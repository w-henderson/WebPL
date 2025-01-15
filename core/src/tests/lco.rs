use crate::Solver;

#[test]
fn lco() {
    let program = r#"
        generate_list(0, []).
        generate_list(N, [N|T]) :- N > 0, N1 is N - 1, generate_list(N1, T).
    "#;

    let query = "generate_list(1000, X).";

    let mut solver = Solver::new(program, query).unwrap();
    assert!(solver.next().is_some());
    assert!(solver.next().is_none());

    // Check that LCO was applied (the stack would've grown much more otherwise)
    assert!(solver.max_choice_point_stack_height() < 50);
}
