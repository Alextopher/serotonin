extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::io::Read;

use pest::{Parser, iterators::Pair};

#[derive(Parser)]
#[grammar = "ut.pest"]
pub struct UTParser;

// AST
#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    // Module name then private definitions then public definitions
    Compound(String, Vec<AstNode>, Vec<AstNode>),

    // Factor
    Atomic(String),
    Natural(u64),
    Byte(u8),
    Term(Vec<AstNode>),

    // simple definition 
    // atomic_symbol ~ integer_constant ~ ":" ~ integer_constant ~ "==" ~ term
    Definition(String, u64, u64, Box<AstNode>),
}

fn parse_term(pair: Pair<Rule>) -> Vec<AstNode> {
    let mut terms: Vec<AstNode> = Vec::new();
    for term in pair.into_inner() {
        match term.as_rule() {
            Rule::term => terms.push(AstNode::Term(parse_term(term))),
            Rule::integer_constant => terms.push(AstNode::Natural(term.as_str().parse().unwrap())),
            Rule::byte_constant => terms.push(AstNode::Byte(term.as_str().parse().unwrap())),
            Rule::atomic_symbol => terms.push(AstNode::Atomic(term.as_str().to_string())),
            _ => unreachable!(),
        }
    }
    terms
}

fn parse_simple_definition(mut rules: pest::iterators::Pairs<Rule>) -> AstNode {
    // First pair is the atomic symbol
    let atomic_symbol = rules.next().unwrap().as_str();

    // Second pair is the number of pops
    let pops = rules.next().unwrap().as_str().parse::<u64>().unwrap();

    // Third pair is the number of pushes
    let pushes = rules.next().unwrap().as_str().parse::<u64>().unwrap();

    // Fourth pair is the term
    let term = parse_term(rules.next().unwrap());

    AstNode::Definition(atomic_symbol.to_string(), pops, pushes, Box::new(AstNode::Term(term)))
}

fn main() {
    // First argument is the file to parse
    let args: Vec<String> = std::env::args().collect();

    // Read the file
    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    
    // Parse the input file
    let pairs = UTParser::parse(Rule::program, &contents).unwrap_or_else(|e| panic!("{}", e));

    // Print the AST
    let mut root: AstNode = AstNode::Atomic("".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::compound_definition => {
                let mut inner = pair.into_inner();
                let module_name = inner.next().unwrap().as_str();
                println!("Module: {}", module_name);
                let mut private_definitions = Vec::new();
                for definition in inner.next().unwrap().into_inner() {
                    match definition.as_rule() {
                        Rule::simple_definition => {
                            private_definitions.push(parse_simple_definition(definition.into_inner()));
                        },
                        _ => {
                            println!("{:?}", definition);
                        }
                    }
                }

                let mut public_definitions = Vec::new();
                for definition in inner.next().unwrap().into_inner() {
                    match definition.as_rule() {
                        Rule::simple_definition => {
                            public_definitions.push(parse_simple_definition(definition.into_inner()));
                        },
                        _ => {
                            println!("{:?}", definition);
                        }
                    }
                }

                root = AstNode::Compound(
                    module_name.to_string(),
                    private_definitions,
                    public_definitions,
                );
            },
            _ => {}
        }
    }

    println!("{:?}", root);
}