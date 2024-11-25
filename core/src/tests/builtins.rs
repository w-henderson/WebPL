use crate::WebPL;

#[test]
fn is() {
    let program = String::new();
    let query = "Y = 3, X is Y + (2 * 5.1).";

    let mut webpl = WebPL::new(program).unwrap();
    let mut solver = webpl.solve(query).unwrap();

    assert_eq!(
        solver.next(),
        Some(vec![("Y".into(), "3".into()), ("X".into(), "13.2".into())])
    );

    assert_eq!(solver.next(), None);
}

#[test]
fn cmp() {
    let program = String::new();
    let query_1 = "4 > 3.";
    let query_2 = "3 > 4.";

    let mut webpl = WebPL::new(program).unwrap();

    let mut solver_1 = webpl.solve(query_1).unwrap();
    assert_eq!(solver_1.next(), Some(vec![]));
    assert_eq!(solver_1.next(), None);

    let mut solver_2 = webpl.solve(query_2).unwrap();
    assert_eq!(solver_2.next(), None);
}
