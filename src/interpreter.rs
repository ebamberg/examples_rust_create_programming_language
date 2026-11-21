use crate::ast::{ComparisonOperator, Expr, Operator, Prefix, TypedExpr};


use std::collections::HashMap;
#[derive(Default)]
pub struct Scope {
    variables: HashMap<String, i32>,
}

pub fn interpret(ast: Vec<TypedExpr>) -> () {
    let mut scope = Scope { variables: std::collections::HashMap::new() };
    ast.iter()
        .map(|node| eval(node, &mut scope))
        .for_each(|result| println!("Result: {}", result));
}

pub fn eval(node: &TypedExpr, scope: &mut Scope) -> i32 {
    match &node.expr {
        Expr::Int(value) => *value,
        Expr::Bool(value) => i32::from(*value),
        Expr::Variable(name) => *scope
            .variables
            .get(name)
            .unwrap_or_else(|| panic!("Undefined variable: {name}")),
        Expr::Assignment { name, value } => {
            let result = eval(value, scope);
            scope.variables.insert(name.clone(), result);
            result
        }
        Expr::Unary { prefix, operand } => {
            let value = eval(operand, scope);
            match prefix {
                Prefix::Plus => value,
                Prefix::Minus => -value,
                Prefix::Not => i32::from(value == 0),
            }
        }
        Expr::Binary { operator, left, right } => {
            let left_value = eval(left, scope);
            let right_value = eval(right, scope);
            match operator {
                Operator::Plus => left_value + right_value,
                Operator::Minus => left_value - right_value,
                Operator::Divide => left_value / right_value,
                Operator::Multiply => left_value * right_value,
            }
        }
        Expr::Comparison { operator, left, right } => {
            let left_value = eval(left, scope);
            let right_value = eval(right, scope);
            i32::from(match operator {
                ComparisonOperator::LessThan => left_value < right_value,
                ComparisonOperator::LessThanOrEqual => left_value <= right_value,
                ComparisonOperator::GreaterThan => left_value > right_value,
                ComparisonOperator::GreaterThanOrEqual => left_value >= right_value,
                ComparisonOperator::Equal => left_value == right_value,
                ComparisonOperator::NotEqual => left_value != right_value,
            })
        }
        Expr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            let branch = if eval(condition, scope) != 0 {
                Some(then_branch)
            } else {
                else_branch.as_ref()
            };
            branch
                .map(|nodes| {
                    nodes
                        .iter()
                        .map(|node| eval(node, scope))
                        .last()
                        .unwrap_or(0)
                })
                .unwrap_or(0)
        }
    }
}
