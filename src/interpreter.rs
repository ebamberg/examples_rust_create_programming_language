use crate::ast::{Node, Operator};

pub fn interpret(ast: Vec<Node>) -> () {
    ast.iter().map(eval).for_each(|result| println!("Result: {}", result));
}

pub fn eval(node: &Node) -> i32 {
    match node {
        Node::Int(value) => *value,
        Node::UnaryExpr { operator, operand } => {
            let value = eval(operand);
            match operator {
                Operator::Plus => value,
                Operator::Minus => -value,
            }
        }
        Node::BinaryExpr { operator, left, right } => {
            let left_value = eval(left);
            let right_value = eval(right);
            match operator {
                Operator::Plus => left_value + right_value,
                Operator::Minus => left_value - right_value,
            }
        }
    }
}
