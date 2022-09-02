use serotonin::{compile, config::Config};
use std::{io::Read, process::exit};

fn main() {
    let app = clap::clap_app!(myapp =>
        (name: "serotonin")
        (version: "0.4.0")
        (author: "Christopher Mahoney")
        (about: "Compiles serotonin to Brainfuck")
        (@arg INPUT: +required +takes_value "serotonin source file")
        (@arg OUTPUT: -o +takes_value "output file")
        (@arg verbose: -v "print verbose output")
        (@arg timings: -t "print timings")
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
        if name.to_string_lossy().contains('.') {
            eprintln!("Error: File name must not contain dots");
            exit(1);
        }
    } else {
        eprintln!("Error: File name must not contain dots");
        exit(1);
    }
    let name = path.file_stem().unwrap().to_string_lossy();

    // Create config from command line flags
    let config = Config::new(matches.is_present("verbose"), matches.is_present("timings"));

    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            match compile(&name, &contents, config) {
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
