use std::collections::HashSet;

use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "ut.pest"]
pub struct UTParser;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Ty {
    pub pops: i64,
    pub pushes: i64,
}

// AST
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AstNode {
    // Module name then private definitions then public definitions
    Compound(String, Vec<AstNode>, Vec<AstNode>),

    // Factor
    Atomic(String),
    Byte(u8),
    If { co: Vec<AstNode>, th: Vec<AstNode>, el: Vec<AstNode> },
    While { co: Vec<AstNode>, body: Vec<AstNode> },

    // simple definition
    // atomic_symbol ~ integer_constant ~ ":" ~ integer_constant ~ "==" ~ term
    Definition(String, Ty, Vec<AstNode>),
}

impl AstNode {
    pub fn get_name(&self) -> String {
        match self {
            AstNode::Compound(name, _, _) => name.clone(),
            AstNode::Atomic(name) => name.clone(),
            AstNode::Definition(name, _, _) => name.clone(),
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
    let mut terms: Vec<Vec<AstNode>> = Vec::new();
    for term in pair.into_inner() {
        match term.as_rule() {
            Rule::term => terms.push(parse_term(term)),
            Rule::integer_constant => terms.push(vec![parse_byte(term)]),
            Rule::atomic_symbol => match term.as_str() {
                "ifte" => {
                    let el = match terms.pop() {
                        Some(el) => el,
                        None => panic!("ifte syntax error: [condition] [then] [else] if"),
                    };

                    let th = match terms.pop() {
                        Some(th) => th,
                        None => panic!("ifte syntax error: [condition] [then] [else] if"),
                    };

                    let co = match terms.pop() {
                        Some(co) => co,
                        None => panic!("ifte syntax error: [condition] [then] [else] if"),
                    };
                    terms.push(vec![AstNode::If { co, el, th }])
                }
                "while" => {
                    let body = match terms.pop() {
                        Some(code) => code,
                        None => panic!("while syntax error: [condition] [code] while"),
                    };

                    let co = match terms.pop() {
                        Some(cond) => cond,
                        None => panic!("while syntax error: [condition] [code] while"),
                    };

                    terms.push(vec![AstNode::While { co, body }])
                }
                _ => terms.push(vec![AstNode::Atomic(term.as_str().to_string())]),
            },
            _ => unreachable!(),
        }
    }

    terms.into_iter().map(Vec::into_iter).flatten().collect()
}

fn parse_simple_definition(pair: Pair<Rule>) -> AstNode {
    let mut rules = pair.into_inner();

    // First pair is the atomic symbol
    let atomic_symbol = rules.next().unwrap().as_str();

    // Second pair is the number of pops
    let pops = rules.next().unwrap().as_str().parse::<i64>().unwrap();

    // Third pair is the number of pushes
    let pushes = rules.next().unwrap().as_str().parse::<i64>().unwrap();

    // Fourth pair is the term
    let term = parse_term(rules.next().unwrap());

    AstNode::Definition(
        atomic_symbol.to_string(),
        Ty { pushes, pops },
        term,
    )
}

fn parse_definitions(pair: Pair<Rule>) -> Vec<AstNode> {
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
            }
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
            }
            _ => {}
        }
    }

    root
}

pub fn parse_single_definition(content: &str) -> AstNode {
    // parse and compile the single function
    let pairs = UTParser::parse(Rule::single_def, content).unwrap_or_else(|e| panic!("{}", e));

    let mut definition = AstNode::Atomic("".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::simple_definition => {
                definition = parse_simple_definition(pair);
            }
            _ => {}
        }
    }

    definition
}
