mod parse;
mod stdlib;
mod gen;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::io::Read;

use parse::parser;

use crate::parse::AstNode;

fn main() {
    // First argument is the file to parse
    let args: Vec<String> = std::env::args().collect();

    // Read the file
    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    
    // compiled functions
    let compiled = &mut stdlib::builtin();

    let root = parser(&contents);

    if let AstNode::Compound(name, private, public) = root {
        // merge the private definitions into the definitions
        for definition in private {
            // compile the definition
            let name = &definition.get_name();
            let code = gen::gen_bf(definition, compiled);
            compiled.insert(name.to_string(), code);
        }

        // merge the public definitions into the definitions
        for definition in public {
            let name = &definition.get_name();
            let code = gen::gen_bf(definition, compiled);
            compiled.insert(name.to_string(), code);
        }

        println!("Loaded \"{}\"", name);
    }

    // Print the main function
    if let Some(code) = compiled.get("main") {
        println!("{}", code);
    } else {
        panic!("No main function defined");
    }
}