use std::collections::HashSet;

use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "ut.pest"]
pub struct UTParser;


// AST
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AstNode {
    // Module name then private definitions then public definitions
    Compound(String, Vec<Box<AstNode>>, Vec<Box<AstNode>>),

    // Factor
    Atomic(String),
    Byte(u8),
    Term(Vec<AstNode>),

    // simple definition 
    // atomic_symbol ~ integer_constant ~ ":" ~ integer_constant ~ "==" ~ term
    Definition(String, u64, u64, Box<AstNode>),
}

impl AstNode {
    pub fn get_name(&self) -> String {
        match self {
            AstNode::Compound(name, _, _) => name.clone(),
            AstNode::Atomic(name) => name.clone(),
            AstNode::Definition(name, _, _, _) => name.clone(),
            _ => unreachable!(),
        }
    }
}

fn parse_byte(pair: Pair<Rule>) -> AstNode {
    // integer is either in the form 0x[0-9a-fA-F]+ or [0-9]+
    let result = if pair.as_str().len() > 2 && pair.as_str()[0..2] == *"0x" {
        let hex = &pair.as_str()[2..];
        u8::from_str_radix(hex, 16).unwrap()
    } else {
        u8::from_str_radix(&pair.as_str(), 10).unwrap()
    };

    AstNode::Byte(result)
}

fn parse_term(pair: Pair<Rule>) -> Vec<AstNode> {
    let mut terms: Vec<AstNode> = Vec::new();
    for term in pair.into_inner() {
        match term.as_rule() {
            Rule::term => terms.push(AstNode::Term(parse_term(term))),
            Rule::integer_constant => terms.push(parse_byte(term)),
            Rule::atomic_symbol => terms.push(AstNode::Atomic(term.as_str().to_string())),
            _ => unreachable!(),
        }
    }
    terms
}

fn parse_simple_definition(pair: Pair<Rule>) -> Box<AstNode> {
    let mut rules = pair.into_inner();

    // First pair is the atomic symbol
    let atomic_symbol = rules.next().unwrap().as_str();

    // Second pair is the number of pops
    let pops = rules.next().unwrap().as_str().parse::<u64>().unwrap();

    // Third pair is the number of pushes
    let pushes = rules.next().unwrap().as_str().parse::<u64>().unwrap();

    // Fourth pair is the term
    let term = parse_term(rules.next().unwrap());

    Box::new(AstNode::Definition(atomic_symbol.to_string(), pops, pushes, Box::new(AstNode::Term(term))))
}

fn parse_definitions(pair: Pair<Rule>) -> Vec<Box<AstNode>> {
    let mut definitions = Vec::new();

    // a hashset to track defined names
    let mut defined_names = HashSet::new();

    for definition in pair.into_inner() {
        match definition.as_rule() {
            Rule::simple_definition => {
                let definition = parse_simple_definition(definition);
                
                // check for duplicate definitions
                if defined_names.contains(&definition.get_name()) {
                    panic!("Duplicate definition \"{}\"", definition.get_name());
                }
                defined_names.insert(definition.get_name());

                // Add definition to the list
                definitions.push(definition);
            },
            _ => unreachable!(),
        }
    }

    definitions
}

fn parse_compound_definition(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();

    let module_name = inner.next().unwrap().as_str();
    let private_definitions = parse_definitions(inner.next().unwrap());
    let public_definitions = parse_definitions(inner.next().unwrap());

    AstNode::Compound(
        module_name.to_string(),
        private_definitions,
        public_definitions,
    )
}

pub fn parser(contents: &str) -> AstNode {
    // Parse the input file
    let pairs = UTParser::parse(Rule::program, &contents).unwrap_or_else(|e| panic!("{}", e));

    let mut root: AstNode = AstNode::Atomic("".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::compound_definition => {
                root = parse_compound_definition(pair);
            },
            _ => {}
        }
    }

    root
}

pub fn parse_single_definition(content: &str) -> Box<AstNode> {
    // parse and compile the single function
    let pairs = UTParser::parse(Rule::single_def, content).unwrap_or_else(|e| panic!("{}", e));

    let mut definition: Box<AstNode> = Box::new(AstNode::Atomic("".to_string()));

    for pair in pairs {
        match pair.as_rule() {
            Rule::simple_definition => {
                definition = parse_simple_definition(pair);
            },
            _ => {}
        }
    }

    definition
}