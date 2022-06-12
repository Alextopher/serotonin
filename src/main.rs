use std::io::Read;
use bfjoy::parser::BFJoyParser;

fn compile(contents: &str, file_name: String) -> String {
    // build the AST
    let mut parser = BFJoyParser::new();

    match parser.module(contents, file_name) {
        Ok(module) => println!("{:?}", module),
        Err(err) => panic!("{}", err),
    };

    String::from("no")
}

fn main() {   
    let mut args = std::env::args();
    let file_name = args.nth(1).unwrap();    
    let mut file = std::fs::File::open(&file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // the module name is the file name without the extension
    let module_name = file_name.split(".").next().unwrap().to_string();

    let main = compile(&contents, module_name);

    if main == "" {
        eprintln!("Missing function \"main\"");
    } else {
        println!("{main}")
    }
}
