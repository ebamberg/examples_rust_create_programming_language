use crate::ast::{ComparisonOperator, Node, Operator, Prefix};

use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::context::Context;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;
use inkwell::types::IntType;
use inkwell::builder::Builder;
use inkwell::IntPredicate;
use std::collections::HashMap;


const ENTRYPOINT_FN_PREFIX: &str = "jit_";

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
    let mut variables = HashMap::new();

    let fn_entrypoint_interface = i32_type.fn_type(&[], false);
    for (index, node) in ast.iter().enumerate() {
        let function_name = format!("{ENTRYPOINT_FN_PREFIX}{index}");
        let fn_entrypoint =
            module.add_function(&function_name, fn_entrypoint_interface, None);
        let entry_basic_block = context.append_basic_block(fn_entrypoint, "entry");
        builder.position_at_end(entry_basic_block);

        let result = eval(&builder, i32_type, node, &module, &mut variables);
        builder.build_return(Some(&result)).unwrap();
    }

    module
}

pub fn print_intermediate_code(module: &Module<'_>) {
    println!(
            "Generated LLVM IR: {}",
            module.print_to_string().to_string()
        );
}


pub fn execute_jit(module: inkwell::module::Module, expression_count: usize) -> Vec<i32> {
    // Placeholder for JIT execution logic
    println!("Executing JIT...");

    // create a JIT execution engine
    let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

    let mut results = Vec::with_capacity(expression_count);
    for index in 0..expression_count {
        let function_name = format!("{ENTRYPOINT_FN_PREFIX}{index}");
        unsafe {
            type EntryFn = unsafe extern "C" fn() -> i32;
            let jit_entrypoint: JitFunction<EntryFn> =
                execution_engine.get_function(&function_name).unwrap();
            let result = jit_entrypoint.call();
            println!("Result of expression {}: {}", index + 1, result);
            results.push(result);
        }
    }

    results
}

fn eval<'ctx>(
    builder: &Builder<'ctx>,
    i32_type: IntType<'ctx>,
    node: &Node,
    module: &Module<'ctx>,
    variables: &mut HashMap<String, PointerValue<'ctx>>,
) -> IntValue<'ctx> {
    match node {
        Node::Int(value) => i32_type.const_int(*value as u64, true),
        Node::Bool(value) => i32_type.const_int(u64::from(*value), false),
        Node::Variable(name) => {
            let pointer = variable_pointer(module, i32_type, variables, name);
            builder
                .build_load(i32_type, pointer, "load_variable")
                .unwrap()
                .into_int_value()
        }
        Node::Assignment { name, value } => {
            let result = eval(builder, i32_type, value, module, variables);
            let pointer = variable_pointer(module, i32_type, variables, name);
            builder.build_store(pointer, result).unwrap();
            result
        }
        Node::UnaryExpr { prefix, operand } => {
            let value = eval(builder, i32_type, operand, module, variables);
            match prefix {
                Prefix::Plus => value,
                Prefix::Minus => value.const_neg(),
                Prefix::Not => {
                    let is_false = builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            value,
                            i32_type.const_zero(),
                            "logical_not",
                        )
                        .unwrap();
                    builder
                        .build_int_z_extend(is_false, i32_type, "logical_not_result")
                        .unwrap()
                }
            }
        }
        Node::BinaryExpr { operator, left, right } => {
            let left_value = eval(builder, i32_type, left, module, variables);
            let right_value = eval(builder, i32_type, right, module, variables);
            match operator {
                Operator::Plus => builder.build_int_add(left_value, right_value, "add_result").unwrap(),
                Operator::Minus => builder.build_int_sub(left_value, right_value, "sub_result").unwrap(),
                Operator::Multiply => builder.build_int_mul(left_value, right_value, "mul_result").unwrap(),
                Operator::Divide => builder.build_int_signed_div(left_value, right_value, "div_result").unwrap(),
            }
        }
        Node::ComparisonExpr { operator, left, right } => {
            let left_value = eval(builder, i32_type, left, module, variables);
            let right_value = eval(builder, i32_type, right, module, variables);
            let predicate = match operator {
                ComparisonOperator::LessThan => IntPredicate::SLT,
                ComparisonOperator::LessThanOrEqual => IntPredicate::SLE,
                ComparisonOperator::GreaterThan => IntPredicate::SGT,
                ComparisonOperator::GreaterThanOrEqual => IntPredicate::SGE,
                ComparisonOperator::Equal => IntPredicate::EQ,
                ComparisonOperator::NotEqual => IntPredicate::NE,
            };
            let comparison = builder
                .build_int_compare(predicate, left_value, right_value, "comparison")
                .unwrap();
            builder
                .build_int_z_extend(comparison, i32_type, "comparison_result")
                .unwrap()
        }
        Node::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition_value = eval(builder, i32_type, condition, module, variables);
            let condition = builder
                .build_int_compare(
                    IntPredicate::NE,
                    condition_value,
                    i32_type.const_zero(),
                    "if_condition",
                )
                .unwrap();
            let function = builder.get_insert_block().unwrap().get_parent().unwrap();
            let then_block = module.get_context().append_basic_block(function, "if_then");
            let else_block = module.get_context().append_basic_block(function, "if_else");
            let merge_block = module.get_context().append_basic_block(function, "if_merge");
            builder
                .build_conditional_branch(condition, then_block, else_block)
                .unwrap();

            builder.position_at_end(then_block);
            let then_value = eval_block(builder, i32_type, then_branch, module, variables);
            let then_exit = builder.get_insert_block().unwrap();
            builder.build_unconditional_branch(merge_block).unwrap();

            builder.position_at_end(else_block);
            let else_value = else_branch
                .as_ref()
                .map(|nodes| eval_block(builder, i32_type, nodes, module, variables))
                .unwrap_or_else(|| i32_type.const_zero());
            let else_exit = builder.get_insert_block().unwrap();
            builder.build_unconditional_branch(merge_block).unwrap();

            builder.position_at_end(merge_block);
            let result = builder.build_phi(i32_type, "if_result").unwrap();
            result.add_incoming(&[(&then_value, then_exit), (&else_value, else_exit)]);
            result.as_basic_value().into_int_value()
        }
    }
}

fn eval_block<'ctx>(
    builder: &Builder<'ctx>,
    i32_type: IntType<'ctx>,
    nodes: &[Node],
    module: &Module<'ctx>,
    variables: &mut HashMap<String, PointerValue<'ctx>>,
) -> IntValue<'ctx> {
    nodes.iter().fold(i32_type.const_zero(), |_, node| {
        eval(builder, i32_type, node, module, variables)
    })
}

fn variable_pointer<'ctx>(
    module: &Module<'ctx>,
    i32_type: IntType<'ctx>,
    variables: &mut HashMap<String, PointerValue<'ctx>>,
    name: &str,
) -> PointerValue<'ctx> {
    *variables.entry(name.to_owned()).or_insert_with(|| {
        let global = module.add_global(i32_type, None, name);
        global.set_initializer(&i32_type.const_zero());
        global.as_pointer_value()
    })
}
