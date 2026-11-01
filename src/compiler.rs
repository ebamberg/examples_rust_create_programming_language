use crate::ast::Node;

use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::context::Context;


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

    
    /* define i32 @add(i32 %x, i32 %y) {
        entry:
        %result = add i32 %x, %y
        ret i32 %result
        } */

    // define i32 @add(i32 %x, i32 %y) {   
    // working with int32 type
    let i32_type = context.i32_type();
    // define function type for add
    let fn_add_interface = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let fn_add = module.add_function("add", fn_add_interface, None);
    // entry:
    let entry_basic_block = context.append_basic_block(fn_add, "entry");
    // position builder at the end of the entry basic block to add commands there
    builder.position_at_end(entry_basic_block);

    // %result = add i32 %x, %y
    let x = fn_add.get_nth_param(0).unwrap().into_int_value();
    let y = fn_add.get_nth_param(1).unwrap().into_int_value();
    // adding the add int+int instruction and returning the result
    let ret = builder.build_int_add(x, y, "result").unwrap();
    // ret i32 %result
    let return_instruction = builder.build_return(Some(&ret)).unwrap();

    module
}


pub fn execute_jit(module: inkwell::module::Module) {
    // Placeholder for JIT execution logic
    println!("Executing JIT...");

    // create a JIT execution engine
    let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

    unsafe {
        // get the function pointer for the add function
        type AddFn = unsafe extern "C" fn(i32, i32) -> i32;
        let add_fn: JitFunction<AddFn> = execution_engine.get_function("add").unwrap();
        // call the add function with sample arguments
        let result = unsafe { add_fn.call(2, 3) };
        println!("Result of add(2, 3): {}", result);
    }
    
}