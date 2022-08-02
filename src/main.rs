use serotonin::parser::compile;
use std::{io::Read, process::exit};

fn main() {
    let app = clap::clap_app!(myapp =>
        (name: "bfjoy")
        (version: "0.2.1")
        (author: "Christopher Mahoney")
        (about: "Compiles bfjoy to Brainfuck")
        (@arg INPUT: +required +takes_value "bfjoy source file")
        (@arg OUTPUT: -o +takes_value "output file")
        (@arg optimize: -O --optimize "optimize generated code for preformance")
        (@arg golf: -g --golf "optimize generated code for length")
    );

    let matches = app.get_matches();

    // Read the file
    let p = matches.value_of("INPUT").unwrap();
    let path = std::path::Path::new(p);
    if !path.exists() {
        eprintln!("Error: File does not exist");
        exit(1);
    }
    // File extension must be .sero
    if let Some(ext) = path.extension() {
        if ext != "sero" {
            eprintln!("Error: File extension must be .sero");
            exit(1);
        }
    } else {
        eprintln!("Error: File extension must be .sero");
        exit(1);
    }
    // File name must not contain dots
    if let Some(name) = path.file_stem() {
        if name.to_string_lossy().contains(".") {
            eprintln!("Error: File name must not contain dots");
            exit(1);
        }
    } else {
        eprintln!("Error: File name must not contain dots");
        exit(1);
    }
    let name = path.file_stem().unwrap().to_string_lossy();

    // Update BFJOY_GOLF and BFJOY_OPTIMIZE environment variables based on the command line arguments
    // let config = if matches.is_present("golf") && matches.is_present("optimize") {
    //     eprintln!("Cannot use both -g and -O");
    //     exit(1);
    // } else if matches.is_present("golf") {
    //     Config {
    //         optimize: false,
    //         golf: true,
    //     }
    // } else if matches.is_present("optimize") {
    //     Config {
    //         optimize: true,
    //         golf: false,
    //     }
    // } else {
    //     Config {
    //         optimize: false,
    //         golf: false,
    //     }
    // };

    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            match compile(&name, &contents) {
                Ok(c) => println!("{}", c),
                Err(e) => {
                    // Report errors
                    eprintln!(
                        "Failed to compile: {}. Found at least {} errors",
                        path.file_name().unwrap().to_string_lossy(),
                        e.len()
                    );

                    for error in e {
                        eprintln!("{}", error);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
