#![allow(clippy::upper_case_acronyms, clippy::result_large_err)]

use pest::{self, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MyLanguageParser;

pub fn parse_source(
    source: &str,
) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    MyLanguageParser::parse(Rule::Program, source)
}

pub fn print_parsed_source(source: &str) {
    match parse_source(source) {
        Ok(pairs) => {
            for pair in pairs {
                println!("Parsed: {:?}", pair);
            }
        }
        Err(e) => {
            eprintln!("Error parsing source: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_source, Rule};

    fn collect_pairs(pair: pest::iterators::Pair<'_, Rule>, output: &mut Vec<(Rule, String)>) {
        if pair.as_rule() != Rule::EOI {
            output.push((pair.as_rule(), pair.as_str().to_owned()));
        }
        for inner in pair.into_inner() {
            collect_pairs(inner, output);
        }
    }

    fn assert_parse_tree(source: &str, expected: &[(Rule, &str)]) {
        let mut actual = Vec::new();
        for pair in parse_source(source).expect("source should parse") {
            collect_pairs(pair, &mut actual);
        }

        let expected: Vec<_> = expected
            .iter()
            .map(|(rule, text)| (*rule, (*text).to_owned()))
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn parses_single_integer() {
        assert_parse_tree(
            "1",
            &[(Rule::Expr, "1"), (Rule::Term, "1"), (Rule::Int, "1")],
        );
    }

    #[test]
    fn parses_positive_unary_integer() {
        assert_parse_tree(
            "+1",
            &[
                (Rule::Expr, "+1"),
                (Rule::UnaryExpr, "+1"),
                (Rule::Operator, "+"),
                (Rule::Term, "1"),
                (Rule::Int, "1"),
            ],
        );
    }

    #[test]
    fn parses_subtraction() {
        assert_parse_tree(
            "2-1",
            &[
                (Rule::Expr, "2-1"),
                (Rule::BinaryExpr, "2-1"),
                (Rule::Term, "2"),
                (Rule::Int, "2"),
                (Rule::Operator, "-"),
                (Rule::Term, "1"),
                (Rule::Int, "1"),
            ],
        );
    }

    #[test]
    fn parses_unary_minus_followed_by_addition() {
        assert_parse_tree(
            "-1+2",
            &[
                (Rule::Expr, "-1+2"),
                (Rule::BinaryExpr, "-1+2"),
                (Rule::UnaryExpr, "-1"),
                (Rule::Operator, "-"),
                (Rule::Term, "1"),
                (Rule::Int, "1"),
                (Rule::Operator, "+"),
                (Rule::Term, "2"),
                (Rule::Int, "2"),
            ],
        );
    }

    #[test]
    fn parses_chain_of_addition_and_subtraction() {
        assert_parse_tree(
            "-1+2+4-7",
            &[
                (Rule::Expr, "-1+2+4-7"),
                (Rule::BinaryExpr, "-1+2+4-7"),
                (Rule::UnaryExpr, "-1"),
                (Rule::Operator, "-"),
                (Rule::Term, "1"),
                (Rule::Int, "1"),
                (Rule::Operator, "+"),
                (Rule::Term, "2"),
                (Rule::Int, "2"),
                (Rule::Operator, "+"),
                (Rule::Term, "4"),
                (Rule::Int, "4"),
                (Rule::Operator, "-"),
                (Rule::Term, "7"),
                (Rule::Int, "7"),
            ],
        );
    }
}
