use pest::error::Error;
use serotonin::{
    bfoptimizer,
    config::Config,
    parser::{compile, Rule},
};
use std::{collections::HashMap, io::Read, process::exit};

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
    let file_name = matches.value_of("INPUT").unwrap();

    // Update BFJOY_GOLF and BFJOY_OPTIMIZE environment variables based on the command line arguments
    let config = if matches.is_present("golf") && matches.is_present("optimize") {
        eprintln!("Cannot use both -g and -O");
        exit(1);
    } else if matches.is_present("golf") {
        Config {
            optimize: false,
            golf: true,
        }
    } else if matches.is_present("optimize") {
        Config {
            optimize: true,
            golf: false,
        }
    } else {
        Config {
            optimize: false,
            golf: false,
        }
    };

    match std::fs::File::open(file_name) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            match compile(&contents) {
                Ok(c) => println!("{}", c),
                Err(e) => println!("{}", e),
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}
