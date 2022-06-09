use std::collections::HashMap;

use crate::parse::AstNode;

// Converts definition to brainfuck
pub fn gen_bf(definition: Box<AstNode>, compiled: &HashMap<String, String>) -> String {
    let mut result = String::new();
    if let AstNode::Definition(_, _, _, term) = *definition {
        if let AstNode::Term(terms) = *term {
            for term in terms {
                match term {
                    AstNode::Byte(byte) => {
                        result.push_str(&number_speed(byte));
                    }
                    AstNode::Atomic(name) => {
                        if let Some(code) = compiled.get(&name) {
                            result.push_str(code);
                        } else {
                            panic!("Undefined symbol: {}", name);
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    result
}

// generates the code to add a number to the stack
// TODO make a code-golfed solution for each number
fn number_speed(n: u8) -> String {
    let mut result = String::from(">");
    for _ in 0..n {
        result.push('+');
    }
    result
}