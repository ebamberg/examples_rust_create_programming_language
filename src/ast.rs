use crate::parser::Rule;
use pest::error::Error;

#[derive(Debug)]
pub enum Type {
    Int,
    Bool,
    Unknown,
}

#[derive(Debug)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Prefix {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
}


impl TypedExpr {
    pub fn new(expr: Expr, ty: Type) -> Self {
        TypedExpr { expr, ty }
    }

    pub fn unknown(expr: Expr) -> Self {
        TypedExpr {
            expr,
            ty: Type::Unknown,
        }
    }
}



#[derive(Debug)]
pub enum Expr {
    Int(i32),
    Bool(bool),
    Variable(String),
    Assignment {
        name: String,
        value: Box<TypedExpr>,
    },
    UnaryExpr {
        prefix: Prefix,
        operand: Box<TypedExpr>,
    },
    BinaryExpr {
        operator: Operator,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    ComparisonExpr {
        operator: ComparisonOperator,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    Conditional {
        condition: Box<TypedExpr>,
        then_branch: Vec<TypedExpr>,
        else_branch: Option<Vec<TypedExpr>>,
    },
}

pub fn from_parsed_pairs(pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<TypedExpr>, Error<Rule>> {
    let mut ast: Vec<TypedExpr> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::Expr => ast.push(expr_to_ast(pair)),
            Rule::Assignment => ast.push(assignment_to_ast(pair)),
            Rule::Conditional => ast.push(conditional_to_ast(pair)),
            Rule::EOI => {}
            rule => {
                return Err(Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: format!("Unexpected rule: {rule:?}"),
                    },
                    pair.as_span(),
                ));
            }
        }
    }

    Ok(ast)
}

fn assignment_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_owned();
    let value = expr_to_ast(inner.next().unwrap());
    TypedExpr::unknown(Expr::Assignment {
        name,
        value: Box::new(value),
    })
}

fn expr_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    match pair.as_rule() {
        Rule::Expr => expr_to_ast(pair.into_inner().next().unwrap()),
        Rule::Comparison => comparison_to_ast(pair),
        Rule::Additive => additive_to_ast(pair),
        Rule::Multiplicative => multiplicative_to_ast(pair),
        Rule::UnaryExpr => unary_expr_to_ast(pair),
        Rule::Primary => primary_to_ast(pair),
        Rule::Int => int_to_ast(pair),
        Rule::Bool => bool_to_ast(pair),
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn comparison_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let mut left = additive_to_ast(inner.next().unwrap());

    while let Some(operator_pair) = inner.next() {
        let operator = comparison_operator_to_ast(operator_pair);
        let right = additive_to_ast(inner.next().unwrap());
        left = TypedExpr::unknown(Expr::ComparisonExpr {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    left
}

fn comparison_operator_to_ast(pair: pest::iterators::Pair<Rule>) -> ComparisonOperator {
    match pair.as_str() {
        "<" => ComparisonOperator::LessThan,
        "<=" => ComparisonOperator::LessThanOrEqual,
        ">" => ComparisonOperator::GreaterThan,
        ">=" => ComparisonOperator::GreaterThanOrEqual,
        "==" => ComparisonOperator::Equal,
        "!=" => ComparisonOperator::NotEqual,
        unknown => panic!("Unknown comparison operator: {unknown}"),
    }
}

fn conditional_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let condition = expr_to_ast(inner.next().unwrap());
    let then_branch = block_to_ast(inner.next().unwrap());
    let else_branch = inner.next().map(block_to_ast);

    TypedExpr::unknown(Expr::Conditional {
        condition: Box::new(condition),
        then_branch,
        else_branch,
    })
}

fn block_to_ast(pair: pest::iterators::Pair<Rule>) -> Vec<TypedExpr> {
    pair.into_inner()
        .map(|pair| match pair.as_rule() {
            Rule::Expr => expr_to_ast(pair),
            Rule::Assignment => assignment_to_ast(pair),
            Rule::Conditional => conditional_to_ast(pair),
            rule => panic!("Unexpected statement in block: {rule:?}"),
        })
        .collect()
}

fn additive_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let mut left = multiplicative_to_ast(inner.next().unwrap());

    while let Some(operator_pair) = inner.next() {
        let operator = operator_to_ast(operator_pair);
        let right = multiplicative_to_ast(inner.next().unwrap());
        left = TypedExpr::unknown(Expr::BinaryExpr {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    left
}

fn multiplicative_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let mut left = unary_expr_to_ast(inner.next().unwrap());

    while let Some(operator_pair) = inner.next() {
        let operator = operator_to_ast(operator_pair);
        let right = unary_expr_to_ast(inner.next().unwrap());
        left = TypedExpr::unknown(Expr::BinaryExpr {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    left
}

fn primary_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    match pair.as_rule() {
        Rule::Primary => {
            let first = pair.into_inner().next().unwrap();
            match first.as_rule() {
                Rule::Int => int_to_ast(first),
                Rule::Bool => bool_to_ast(first),
                Rule::Literal => primary_to_ast(first),
                Rule::Identifier => TypedExpr::unknown(Expr::Variable(first.as_str().to_owned())),
                Rule::Expr => expr_to_ast(first),
                unknown => panic!("Unknown primary: {:?}", unknown),
            }
        }
        Rule::Literal => {
            let literal = pair.into_inner().next().unwrap();
            match literal.as_rule() {
                Rule::Int => int_to_ast(literal),
                Rule::Bool => bool_to_ast(literal),
                unknown => panic!("Unknown literal: {:?}", unknown),
            }
        }
        Rule::Int => int_to_ast(pair),
        Rule::Bool => bool_to_ast(pair),
        Rule::Expr => expr_to_ast(pair),
        unknown => panic!("Unknown primary: {:?}", unknown),
    }
}

fn unary_expr_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    if first.as_rule() == Rule::Prefix {
        let prefix = prefix_to_ast(first);
        let operand = primary_to_ast(inner.next().unwrap());
        TypedExpr::unknown(Expr::UnaryExpr {
            prefix,
            operand: Box::new(operand),
        })
    } else {
        primary_to_ast(first)
    }
}


fn operator_to_ast(pair: pest::iterators::Pair<Rule>) -> Operator {
    match pair.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "/" => Operator::Divide,
        unknown => panic!("Unknown operator: {unknown}"),
    }
}

fn prefix_to_ast(pair: pest::iterators::Pair<Rule>) -> Prefix {
    match pair.as_str() {
        "+" => Prefix::Plus,
        "-" => Prefix::Minus,
        "!" => Prefix::Not,
        unknown => panic!("Unknown prefix: {unknown}"),
    }
}

fn int_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    let int: i32 = pair.as_str().parse().unwrap();
    TypedExpr::new(Expr::Int(int), Type::Int)
}

fn bool_to_ast(pair: pest::iterators::Pair<Rule>) -> TypedExpr {
    TypedExpr::new(Expr::Bool(pair.as_str() == "true"), Type::Bool)
}

pub fn print_ast(ast: &[TypedExpr]) {
    println!("AST");
    for (index, node) in ast.iter().enumerate() {
        print_node(node, "", index == ast.len() - 1);
    }
}

fn print_node(node: &TypedExpr, node_prefix: &str, is_last: bool) {
    let branch = if is_last { "\\-- " } else { "|-- " };
    match &node.expr {
        Expr::Int(value) => println!("{node_prefix}{branch}Int({value})"),
        Expr::Bool(value) => println!("{node_prefix}{branch}Bool({value})"),
        Expr::Variable(name) => println!("{node_prefix}{branch}Variable({name})"),
        Expr::Assignment { name, value } => {
            println!("{node_prefix}{branch}Assignment({name})");
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            print_node(value, &child_prefix, true);
        }
        Expr::UnaryExpr { prefix, operand } => {
            println!("{node_prefix}{branch}UnaryExpr({})", prefix_name(prefix));
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            print_node(operand, &child_prefix, true);
        }
        Expr::BinaryExpr {
            operator,
            left,
            right,
        } => {
            println!("{node_prefix}{branch}BinaryExpr({})", operator_name(operator));
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            println!("{child_prefix}|-- Left");
            let left_prefix = format!("{child_prefix}|   ");
            print_node(left, &left_prefix, true);
            println!("{child_prefix}\\-- Right");
            let right_prefix = format!("{child_prefix}    ");
            print_node(right, &right_prefix, true);
        }
        Expr::ComparisonExpr {
            operator,
            left,
            right,
        } => {
            println!("{node_prefix}{branch}ComparisonExpr({})", comparison_operator_name(operator));
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            print_node(left, &format!("{child_prefix}|   "), true);
            print_node(right, &format!("{child_prefix}    "), true);
        }
        Expr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            println!("{node_prefix}{branch}Conditional");
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            print_node(condition, &child_prefix, true);
            for node in then_branch {
                print_node(node, &child_prefix, true);
            }
            if let Some(nodes) = else_branch {
                for node in nodes {
                    print_node(node, &child_prefix, true);
                }
            }
        }
    }
}

fn operator_name(operator: &Operator) -> &'static str {
    match operator {
        Operator::Plus => "+",
        Operator::Minus => "-",
        Operator::Divide => "/",
        Operator::Multiply => "*",
    }
}

fn comparison_operator_name(operator: &ComparisonOperator) -> &'static str {
    match operator {
        ComparisonOperator::LessThan => "<",
        ComparisonOperator::LessThanOrEqual => "<=",
        ComparisonOperator::GreaterThan => ">",
        ComparisonOperator::GreaterThanOrEqual => ">=",
        ComparisonOperator::Equal => "==",
        ComparisonOperator::NotEqual => "!=",
    }
}

fn prefix_name(prefix: &Prefix) -> &'static str {
    match prefix {
        Prefix::Plus => "+",
        Prefix::Minus => "-",
        Prefix::Not => "!",
    }
}

