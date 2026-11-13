mod ast;
mod compiler;
mod interpreter;
mod parser;

#[cfg(test)]
mod tests;

use ast::{from_parsed_pairs, print_ast};
use compiler::{compile, execute_jit, print_intermediate_code};
use parser::parse_source;
use std::{env, error::Error, ffi::OsString, fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Simple Calculator parsing example!");

    let source = read_source()?;
    let parsed_source = parse_source(&source).expect("Syntax Error");
    let ast_tree = from_parsed_pairs(parsed_source).expect("AST conversion failed");
    print_ast(&ast_tree);
    println!("{:#?}", ast_tree);
    //   interpreter::interpret(ast_tree);
    let expression_count = ast_tree.len();
    let compiler_context = compiler::new_compiler_context();
    let module = compile(&compiler_context, ast_tree);
    print_intermediate_code(&module);
    execute_jit(module, expression_count);

    Ok(())
}

fn read_source() -> io::Result<String> {
    read_source_from(env::args_os().nth(1))
}

fn read_source_from(filename: Option<OsString>) -> io::Result<String> {
    let filename = filename.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/testlang.lang")
    });
    fs::read_to_string(filename)
}
