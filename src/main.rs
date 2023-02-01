use clap::{arg, command, crate_version, value_parser, Arg, ArgAction, ArgMatches};
use serotonin::{compile, config::Config};
use std::{
    borrow::Cow,
    fs,
    io::{self, stdin, BufRead, Read, Write},
    num::Wrapping,
    path::PathBuf,
    process::exit,
    thread,
};

fn compile_from_matches(matches: &ArgMatches) -> String {
    let (name, contents) = match matches.get_one::<PathBuf>("INPUTS") {
        Some(path) => {
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

            match std::fs::File::open(path) {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    (name, contents)
                }
                Err(e) => {
                    eprintln!("{e}");
                    exit(1);
                }
            }
        }
        None => {
            let contents = stdin()
                .lock()
                .lines()
                .filter_map(|r| r.ok())
                .collect::<String>();
            let name: Cow<str> = "main".into();

            (name, contents)
        }
    };

    // Create config from command line flags
    let config = Config::new(
        matches.is_present("verbose"),
        matches.is_present("timings"),
        !matches.is_present("no optimize"),
    );

    match compile(&name, &contents, config) {
        Ok(c) => c,
        Err(e) => {
            // Report errors
            eprintln!(
                "Failed to compile: {}. Found at least {} errors",
                name,
                e.len()
            );

            for error in e {
                eprintln!("{}", error);
            }

            exit(1);
        }
    }
}

/// The `build` subcommand
fn build(matches: &ArgMatches) -> io::Result<()> {
    let code = compile_from_matches(matches);

    match matches.get_one::<PathBuf>("OUTPUT") {
        Some(p) => fs::write(p, code)?,
        None => println!("{code}"),
    }

    Ok(())
}

/// The `run` subcommand
fn run(matches: &ArgMatches) -> io::Result<()> {
    let code = compile_from_matches(matches);

    match bfi::spawn(&code, u64::max_value()) {
        Ok((tx, rx, join)) => {
            let out = if matches.get_flag("raw") {
                // On one thread read from stdin
                thread::spawn(move || {
                    // lock stdin
                    let mut stdin = io::stdin().lock();

                    loop {
                        let mut buffer = String::new();
                        stdin.read_line(&mut buffer).unwrap();
                        buffer
                            .split_whitespace()
                            .map(|s| s.parse())
                            .filter(|b| b.is_ok())
                            .for_each(|b| tx.send(Wrapping(b.unwrap())).unwrap())
                    }
                });

                // On the another write to stdout
                thread::spawn(move || {
                    let mut stdout = io::stdout().lock();
                    while let Ok(b) = rx.recv() {
                        match b {
                            Ok(b) => {
                                write!(stdout, "{} ", b).unwrap();
                            }
                            Err(err) => {
                                eprintln!("Runtime Error {:?}", err);
                                exit(1);
                            }
                        }
                    }
                    stdout.flush().unwrap();
                })
            } else {
                // On one thread read from stdin
                thread::spawn(move || {
                    // lock stdin
                    let mut stdin = io::stdin().lock();

                    loop {
                        let mut buffer = String::new();
                        stdin.read_line(&mut buffer).unwrap();
                        buffer.bytes().for_each(|b| tx.send(Wrapping(b)).unwrap())
                    }
                });

                // On the another write to stdout
                thread::spawn(move || {
                    let mut stdout = io::stdout().lock();
                    let mut buf: [u8; 1] = [0];
                    while let Ok(b) = rx.recv() {
                        match b {
                            Ok(b) => {
                                buf[0] = b.0;
                                stdout.write_all(&buf).unwrap();
                            }
                            Err(err) => {
                                eprintln!("Runtime Error {:?}", err);
                                exit(1);
                            }
                        }
                    }
                    stdout.flush().unwrap();
                })
            };

            join.join().unwrap();
            out.join().unwrap();
        }
        Err(err) => {
            eprintln!("{err:?}");
            exit(1);
        }
    }

    Ok(())
}

fn main() {
    let build_args = &[
        Arg::new("no optimize")
            .long("--no-optimize")
            .help("Disable optimizations"),
        arg!(-t --timings "Print timing information"),
        arg!(-v --verbose "Print verbose outputs"),
        arg!([INPUTS] "serotonin source files").value_parser(value_parser!(PathBuf)),
    ];

    let matches = command!()
        .version(crate_version!())
        .author("Christopher Mahoney @Alextopher")
        .subcommand(
            command!("build")
                .about("Compiles serotonin to Brainfuck")
                .arg(
                    arg!(-o [OUTPUT] "Save brainfuck output to a file")
                        .max_values(1)
                        .value_parser(value_parser!(PathBuf)),
                )
                .args(build_args),
        )
        .subcommand(
            command!("run")
                .about("Compiles serotonin source files to Brainfuck and executes it with `bfi`")
                .arg(arg!(-r --raw "Raw interpreter in raw mode").action(ArgAction::SetTrue))
                .args(build_args),
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("build", m)) => build(m),
        Some(("run", m)) => run(m),
        _ => Ok(()),
    };

    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    }
}
