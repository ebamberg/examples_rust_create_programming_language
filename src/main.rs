mod ast;
mod parser;
mod interpreter;
mod compiler;

use ast::{from_parsed_pairs, print_ast};
use parser::parse_source;
use interpreter::interpret;
use compiler::{compile,execute_jit};

fn main() {
    println!("Simple Calculator parsing example!");

    let source = "-1+(2-3)";
    let parsed_source = parse_source(source).expect("Syntax Error");
    let ast_tree = from_parsed_pairs(parsed_source).expect("AST conversion failed");
    print_ast(&ast_tree);
 //   interpret(ast_tree);
    let compiler_context = compiler::new_compiler_context();
    let module = compile(&compiler_context, ast_tree);
    execute_jit(module);

}
