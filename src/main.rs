use std::io::Read;
use untitled::{parse, gen::Gen};

fn compile(contents: &str) -> String {
    // add built-in functions
    let mut gen = Gen::builtins();

    // build the AST
    let compound = parse::parser(contents);

    // generate all the functions in compound and return "main"
    gen.gen(compound)
}

fn main() {   
    let mut args = std::env::args();
    let file_name = args.nth(1).unwrap();    
    let mut file = std::fs::File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let main = compile(&contents);

    if main == "" {
        eprintln!("Missing function \"main\"");
    } else {
        println!("{main}")
    }
}
