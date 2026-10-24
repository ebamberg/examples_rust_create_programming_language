mod ast;
mod parser;
mod interpreter;

use ast::{from_parsed_pairs, print_ast};
use parser::parse_source;
use interpreter::interpret;

fn main() {
    println!("Simple Calculator parsing example!");

    let source = "-1+(2-3)";
    let parsed_source = parse_source(source).expect("Syntax Error");
    let ast_tree = from_parsed_pairs(parsed_source).expect("AST conversion failed");
    print_ast(&ast_tree);
    interpret(ast_tree);

}
