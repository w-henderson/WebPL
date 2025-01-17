use crate::{Error, ErrorLocation, Solver};

#[test]
fn invalid_token() {
    let program = r#"
        a(X) :- b(X).
        b(X) :- @ c(X).
    "#;

    if let Err(e) = Solver::new(program, "a(3).") {
        assert_eq!(
            e,
            Error {
                location: Some(ErrorLocation {
                    offset: 39,
                    line: 3,
                    column: 17
                }),
                error: "Invalid token".into()
            }
        );
    } else {
        panic!("Expected an error");
    }
}

#[test]
fn unexpected_token() {
    let program = r#"
        a(X) :- b(X).
        b(X) :- hello c(X).
    "#;

    if let Err(e) = Solver::new(program, "a(3).") {
        assert_eq!(
            e,
            Error {
                location: Some(ErrorLocation {
                    offset: 45,
                    line: 3,
                    column: 23
                }),
                error: "Unexpected token `c`".into()
            }
        );
    } else {
        panic!("Expected an error");
    }
}

#[test]
fn unexpected_eof() {
    let program = r#"
        a(X) :- b(X).
        b(3)
    "#;

    if let Err(e) = Solver::new(program, "a(3).") {
        assert_eq!(
            e,
            Error {
                location: Some(ErrorLocation {
                    offset: 35,
                    line: 3,
                    column: 13
                }),
                error: "Unexpected end of file, did you forget a '.'?".into()
            }
        );
    } else {
        panic!("Expected an error");
    }
}
