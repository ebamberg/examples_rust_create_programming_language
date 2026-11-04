use crate::{ast::from_parsed_pairs, interpreter, parser::parse_source};

fn eval(source: &str) -> i32 {
    let parsed = parse_source(source).expect("source should parse");
    let mut ast = from_parsed_pairs(parsed).expect("AST conversion should succeed");
    assert_eq!(ast.len(), 1, "expected one expression");
    interpreter::eval(&ast.remove(0))
}

#[test]
fn test_addition() {
    assert_eq!(eval("1 + 2"), 3);
}

#[test]
fn test_subtraction() {
    assert_eq!(eval("5 - 3"), 2);
}

#[test]
fn test_combined() {
    assert_eq!(eval("1 + 2 - 3"), 0);
}