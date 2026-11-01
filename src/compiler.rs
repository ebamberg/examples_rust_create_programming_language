use crate::ast::{Node,Operator};

use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::context::Context;
use inkwell::values::IntValue;
use inkwell::types::IntType;
use inkwell::builder::Builder;


const ENTRYPOINT_FN_NAME: &str = "jit";

pub fn new_compiler_context() -> Context {
    // Placeholder for creating a new compiler context
    println!("Creating new compiler context...");
    Context::create()
}

// <'ctx> is a lifetime parameter , Context and Module are generic over this lifetime, meaning they can hold references that live at least as long as 'ctx. This is important for ensuring that the LLVM context and module are valid for the duration of their use in the compilation process.
pub fn compile<'ctx>(context: &'ctx Context, ast: Vec<Node>) -> Module<'ctx> {
    // Placeholder for compilation logic
    println!("Compiling AST...");

    // init llvm inkwell context, module, and builder
    let module = context.create_module("main_module");
    let builder = context.create_builder(); 

    
    /* define i32 @jit() {
        entry:
            ... compiled code
        ret i32 %result
        } */

    // define i32 @add(i32 %x, i32 %y) {   
    // working with int32 type
    let i32_type = context.i32_type();

    let fn_entrypoint_interface = i32_type.fn_type(&[], false);
    let fn_entrypoint = module.add_function(ENTRYPOINT_FN_NAME, fn_entrypoint_interface, None);
    // entry:
    let entry_basic_block = context.append_basic_block(fn_entrypoint, "entry");
    // position builder at the end of the entry basic block to add commands there
    builder.position_at_end(entry_basic_block);

    //@TODO: multiple lines results in multiple ret statements, need to handle this better
    ast.iter().map(|node| eval(&builder, &i32_type, node) ).for_each(|result| {
        // ret i32 %result
        builder.build_return(Some(&result)).unwrap();
    });

    module
}

pub fn print_intermediate_code(module: &Module<'_>) {
    println!(
            "Generated LLVM IR: {}",
            module.print_to_string().to_string()
        );
}


pub fn execute_jit(module: inkwell::module::Module) {
    // Placeholder for JIT execution logic
    println!("Executing JIT...");

    // create a JIT execution engine
    let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

    unsafe {
        // get the function pointer for the add function
        type EntryFn = unsafe extern "C" fn() -> i32;
        let jit_entrypoint: JitFunction<EntryFn> = execution_engine.get_function(ENTRYPOINT_FN_NAME).unwrap();
        // call the add function with sample arguments
        let result = jit_entrypoint.call();
        println!("Result of expression: {}", result);
    }
    
}

fn eval<'ctx>(builder: &'ctx Builder, i32_type: &'ctx IntType, node: &Node) -> IntValue<'ctx> {
    match node {
        Node::Int(value) => i32_type.const_int(*value as u64, true),
        Node::UnaryExpr { operator, operand } => {
            let value = eval(builder, i32_type, operand);
            match operator {
                Operator::Plus => value,
                Operator::Minus => value.const_neg(),
            }
        }
        Node::BinaryExpr { operator, left, right } => {
            let left_value = eval(builder, i32_type,left);
            let right_value = eval(builder, i32_type, right);
            match operator {
                Operator::Plus => builder.build_int_add(left_value, right_value, "add_result").unwrap(),
                Operator::Minus => builder.build_int_sub(left_value, right_value, "sub_result").unwrap(),
            }
        }
    }
}