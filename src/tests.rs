use crate::{ast::from_parsed_pairs, compiler, interpreter, parser::parse_source};
use std::{ffi::OsString, fs, io::ErrorKind};

#[test]
fn reads_default_example_file() {
    crate::read_source_from(None).expect("default example should be readable");
}

#[test]
fn reads_filename_from_argument() {
    let filename = std::env::temp_dir().join(format!(
        "simple-language-test-input-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    let expected = "17+25";
    fs::write(&filename, expected).expect("temporary input file should be created");
    let source = crate::read_source_from(Some(OsString::from(&filename)))
        .expect("argument filename should be readable");
    fs::remove_file(filename).expect("temporary input file should be removed");
    assert_eq!(source, expected);
}

#[test]
fn default_program_executes_every_expression() {
    let source = "3*4+6/3;\n10-4*2";
    let parsed = parse_source(&source).expect("default example should parse");
    let ast = from_parsed_pairs(parsed).expect("default example should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);

    let results = compiler::execute_jit(module, expression_count);

    assert_eq!(results, [14, 2]);
}

#[test]
fn missing_filename_returns_not_found() {
    let filename = std::env::temp_dir().join(format!(
        "simple-language-test-missing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    let error = crate::read_source_from(Some(filename.into_os_string()))
        .expect_err("missing filename should return an error");
    assert_eq!(error.kind(), ErrorKind::NotFound);
}

#[cfg(unix)]
#[test]
fn unreadable_file_returns_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let filename = std::env::temp_dir().join(format!(
        "simple-language-test-unreadable-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos()
    ));
    fs::write(&filename, "1").expect("temporary file should be created");
    fs::set_permissions(&filename, fs::Permissions::from_mode(0o000))
        .expect("temporary file should become unreadable");

    let result = crate::read_source_from(Some(OsString::from(&filename)));
    fs::set_permissions(&filename, fs::Permissions::from_mode(0o600))
        .expect("temporary file permissions should be restored");
    fs::remove_file(filename).expect("temporary file should be removed");

    match result {
        Err(error) => assert_eq!(error.kind(), ErrorKind::PermissionDenied),
        Ok(_) => eprintln!("Permission check skipped: this process can read mode-000 files"),
    }
}

fn eval(source: &str) -> i32 {
    let parsed = parse_source(source).expect("source should parse");
    let ast = from_parsed_pairs(parsed).expect("AST conversion should succeed");
    assert_eq!(ast.len(), 1, "expected one expression");
    let mut scope = interpreter::Scope::default();
    interpreter::eval(&ast[0], &mut scope)
}

fn eval_program(source: &str) -> Vec<i32> {
    let parsed = parse_source(source).expect("source should parse");
    let ast = from_parsed_pairs(parsed).expect("AST conversion should succeed");
    let mut scope = interpreter::Scope::default();
    ast.iter()
        .map(|node| interpreter::eval(node, &mut scope))
        .collect()
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

#[test]
fn test_multiplication_precedence() {
    assert_eq!(eval("2+3*4"), 14);
}

#[test]
fn parses_assignment_into_ast() {
    let parsed = parse_source("let a=1").expect("assignment should parse");
    let ast = from_parsed_pairs(parsed).expect("assignment should convert to an AST");

    assert_eq!(ast.len(), 1);
    match &ast[0] {
        crate::ast::Node::Assignment { name, value } => {
            assert_eq!(name, "a");
            assert!(matches!(value.as_ref(), crate::ast::Node::Int(1)));
        }
        node => panic!("expected assignment node, got {node:?}"),
    }
}

#[test]
fn interpreter_resolves_variables_in_expressions_and_assignments() {
    assert_eq!(eval_program("let a=3; let b=a*2; b+a"), [3, 6, 9]);
}

#[test]
fn interpreter_executes_variable_assignment_program() {
    let source = "let a=1;\nlet b=a*2;\nb;";

    assert_eq!(eval_program(source), [1, 2, 2]);
}

#[test]
fn comparison_result_can_be_assigned_to_a_variable() {
    assert_eq!(eval_program("let a=3; let below=a<10; below"), [3, 1, 1]);
}

#[test]
fn boolean_literal_can_be_assigned_to_a_variable() {
    assert_eq!(eval_program("let enabled=true; enabled; let disabled=false; disabled"), [1, 1, 0, 0]);
}

#[test]
fn conditional_executes_the_matching_block_with_or_without_else() {
    let with_else = eval_program(
        "let a=3; if (a<10) { let result=7; result; } else { 4; }",
    );
    let without_else = eval_program("let a=3; if (a>10) { 8; }");

    assert_eq!(with_else, [3, 7]);
    assert_eq!(without_else, [3, 0]);
}

#[test]
fn conditional_compares_boolean_variables_with_true() {
    let equal_true = eval_program(
        "let a=true;\nif (a==true) {\n    let b=2;\n} else {\n    let b=4;\n};\nb;",
    );
    let not_equal_true = eval_program(
        "let a=true;\nif (a!=true) {\n    let b=2;\n} else {\n    let b=4;\n};\nb;",
    );

    assert_eq!(equal_true, [1, 2, 2]);
    assert_eq!(not_equal_true, [1, 4, 4]);
}

#[test]
fn conditional_accepts_truthy_and_negated_variables() {
    let truthy = eval_program("let a=true; if (a) { 2; } else { 4; }");
    let negated_false = eval_program("let a=false; if (!a) { 2; } else { 4; }");

    assert_eq!(truthy, [1, 2]);
    assert_eq!(negated_false, [0, 2]);
}

#[test]
fn conditional_compares_negated_variable_to_integer() {
    let source = "let a=20;\nif (!a<10) {\n    let b=2;\n} else {\n    let b=4;\n};\nb;";
    let interpreter_results = eval_program(source);

    let parsed = parse_source(source).expect("program should parse");
    let ast = from_parsed_pairs(parsed).expect("program should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);
    let jit_results = compiler::execute_jit(module, expression_count);

    assert_eq!(interpreter_results, [20, 2, 2]);
    assert_eq!(jit_results, [20, 2, 2]);
}

#[test]
fn jit_evaluates_comparison_assignment() {
    let parsed = parse_source("let a=3; let below=a<10; below")
        .expect("program should parse");
    let ast = from_parsed_pairs(parsed).expect("program should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);

    assert_eq!(compiler::execute_jit(module, expression_count), [3, 1, 1]);
}

#[test]
fn jit_executes_conditional_block() {
    let parsed = parse_source("let a=3; if (a<10) { 7; } else { 4; }")
        .expect("program should parse");
    let ast = from_parsed_pairs(parsed).expect("program should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);

    assert_eq!(compiler::execute_jit(module, expression_count), [3, 7]);
}

#[test]
fn jit_executes_truthy_and_negated_conditions() {
    let parsed = parse_source("let a=false; if (!a) { 2; } else { 4; }")
        .expect("program should parse");
    let ast = from_parsed_pairs(parsed).expect("program should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);

    assert_eq!(compiler::execute_jit(module, expression_count), [0, 2]);
}

#[test]
fn jit_resolves_variables_across_statements() {
    let parsed = parse_source("let a=3; let b=a*2; b+a").expect("program should parse");
    let ast = from_parsed_pairs(parsed).expect("program should convert to an AST");
    let expression_count = ast.len();
    let context = compiler::new_compiler_context();
    let module = compiler::compile(&context, ast);

    let results = compiler::execute_jit(module, expression_count);

    assert_eq!(results, [3, 6, 9]);
}
