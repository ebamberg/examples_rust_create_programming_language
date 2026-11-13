use crate::{ast::from_parsed_pairs, compiler, interpreter, parser::parse_source};
use std::{ffi::OsString, fs, io::ErrorKind, path::PathBuf};

#[test]
fn reads_default_example_file() {
    let source = crate::read_source_from(None).expect("default example should be readable");
    assert_eq!(source.trim(), "3*4+6/3;\n10-4*2");
}

#[test]
fn reads_filename_from_argument() {
    let filename = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/testlang.lang");
    let source = crate::read_source_from(Some(filename.into_os_string()))
        .expect("argument filename should be readable");
    assert_eq!(source.trim(), "3*4+6/3;\n10-4*2");
}

#[test]
fn default_program_executes_every_expression() {
    let source = crate::read_source_from(None).expect("default example should be readable");
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

#[test]
fn test_multiplication_precedence() {
    assert_eq!(eval("2+3*4"), 14);
}
