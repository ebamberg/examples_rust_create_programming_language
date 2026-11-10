use crate::parser::Rule;
use pest::error::Error;

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
}



#[derive(Debug)]
pub enum Node {
    Int(i32),
    UnaryExpr {
        prefix: Prefix,
        operand: Box<Node>,
    },
    BinaryExpr {
        operator: Operator,
        left: Box<Node>,
        right: Box<Node>,
    },
}

pub fn from_parsed_pairs(pairs: pest::iterators::Pairs<Rule>) -> Result<Vec<Node>, Error<Rule>> {
    let mut ast: Vec<Node> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::Expr => ast.push(expr_to_ast(pair)),
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

fn expr_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Expr => expr_to_ast(pair.into_inner().next().unwrap()),
        Rule::Additive => additive_to_ast(pair),
        Rule::Multiplicative => multiplicative_to_ast(pair),
        Rule::UnaryExpr => unary_expr_to_ast(pair),
        Rule::Primary => primary_to_ast(pair),
        Rule::Int => int_to_ast(pair),
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn additive_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    let mut left = multiplicative_to_ast(inner.next().unwrap());

    while let Some(operator_pair) = inner.next() {
        let operator = operator_to_ast(operator_pair);
        let right = multiplicative_to_ast(inner.next().unwrap());
        left = Node::BinaryExpr {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    left
}

fn multiplicative_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    let mut inner = pair.into_inner();
    let mut left = unary_expr_to_ast(inner.next().unwrap());

    while let Some(operator_pair) = inner.next() {
        let operator = operator_to_ast(operator_pair);
        let right = unary_expr_to_ast(inner.next().unwrap());
        left = Node::BinaryExpr {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    left
}

fn primary_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Primary => {
            let first = pair.into_inner().next().unwrap();
            match first.as_rule() {
                Rule::Int => int_to_ast(first),
                Rule::Expr => expr_to_ast(first),
                unknown => panic!("Unknown primary: {:?}", unknown),
            }
        }
        Rule::Int => int_to_ast(pair),
        Rule::Expr => expr_to_ast(pair),
        unknown => panic!("Unknown primary: {:?}", unknown),
    }
}

fn unary_expr_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    let rule = pair.as_rule();
    let mut inner = pair.into_inner();

    match rule {
        Rule::Prefix => {
            let prefix = prefix_to_ast(inner.next().unwrap());
            let operand = primary_to_ast(inner.next().unwrap());
            Node::UnaryExpr {
                prefix,
                operand: Box::new(operand),
            }
        }
        _ => primary_to_ast(inner.next().unwrap()),
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
        unknown => panic!("Unknown prefix: {unknown}"),
    }
}

fn int_to_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    let int: i32 = pair.as_str().parse().unwrap();
    Node::Int(int)
}

pub fn print_ast(ast: &[Node]) {
    println!("AST");
    for (index, node) in ast.iter().enumerate() {
        print_node(node, "", index == ast.len() - 1);
    }
}

fn print_node(node: &Node, node_prefix: &str, is_last: bool) {
    let branch = if is_last { "\\-- " } else { "|-- " };
    match node {
        Node::Int(value) => println!("{node_prefix}{branch}Int({value})"),
        Node::UnaryExpr { prefix, operand } => {
            println!("{node_prefix}{branch}UnaryExpr({})", prefix_name(prefix));
            let child_prefix = format!("{node_prefix}{}   ", if is_last { "    " } else { "|" });
            print_node(operand, &child_prefix, true);
        }
        Node::BinaryExpr {
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

fn prefix_name(prefix: &Prefix) -> &'static str {
    match prefix {
        Prefix::Plus => "+",
        Prefix::Minus => "-",
    }
}

