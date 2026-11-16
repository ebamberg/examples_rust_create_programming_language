use crate::ast::{Node, Operator, Prefix};


use std::collections::HashMap;
#[derive(Default)]
pub struct Scope {
    variables: HashMap<String, i32>,
}

pub fn interpret(ast: Vec<Node>) -> () {
    let mut scope = Scope { variables: std::collections::HashMap::new() };
    ast.iter().map(|node| eval(node, &mut scope)).for_each(|result| println!("Result: {}", result));
}

pub fn eval(node: &Node, scope: &mut Scope) -> i32 {
    match node {
        Node::Int(value) => *value,
        Node::Variable(name) => *scope
            .variables
            .get(name)
            .unwrap_or_else(|| panic!("Undefined variable: {name}")),
        Node::Assignment { name, value } => {
            let result = eval(value, scope);
            scope.variables.insert(name.clone(), result);
            result
        }
        Node::UnaryExpr { prefix, operand } => {
            let value = eval(operand,scope);
            match prefix {
                Prefix::Plus => value,
                Prefix::Minus => -value,
            }
        }
        Node::BinaryExpr { operator, left, right } => {
            let left_value = eval(left,scope);
            let right_value = eval(right,scope);
            match operator {
                Operator::Plus => left_value + right_value,
                Operator::Minus => left_value - right_value,
                Operator::Divide => left_value / right_value,
                Operator::Multiply => left_value * right_value,
            }
        }
    }
}
