use std::{io::Read, collections::HashMap};
use untitled::{parse::{parser, AstNode}, stdlib, gen::gen_bf};

fn compile(contents: &str) -> HashMap<String, String> {
    // add built-in functions
    let mut compiled = stdlib::builtin();

    // compile the standard library
    stdlib::load_lib(&mut compiled);

    // build the AST
    let root = parser(&contents);
    if let AstNode::Compound(_name, private, public) = root {
        // merge the private definitions into the definitions
        for definition in private {
            // compile the definition
            let name = &definition.get_name();
            let code = gen_bf(definition, &compiled);
            compiled.insert(name.to_string(), code);
        }

        // merge the public definitions into the definitions
        for definition in public {
            let name = &definition.get_name();
            let code = gen_bf(definition, &compiled);
            compiled.insert(name.to_string(), code);
        }
    }

    compiled
}

fn main() {   
    let mut args = std::env::args();
    let file_name = args.nth(1).unwrap();    
    let mut file = std::fs::File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let compiled = compile(&contents);

    // Print the main function
    if let Some(code) = compiled.get("main") {
        println!("{}", code);
    } else {
        panic!("No main function defined");
    }
}
