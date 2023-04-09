use bfi::TestResults;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

fn multiple_test(code: &str, inputs: Vec<Vec<u8>>, outputs: Vec<Vec<u8>>) {
    assert!(inputs.len() == outputs.len());

    // compile
    match compile_without_timings("test", code, Config::default()) {
        Ok(code) => {
            println!("{}", code);

            // execute the program
            match bfi::tests_blocking(
                &code,
                inputs.into_iter(),
                outputs.into_iter(),
                MAX_ITERATIONS,
            ) {
                TestResults::OutputsDontMatchInputs => {}
                TestResults::ParseError(err) => {
                    panic!("{:?}", err);
                }
                TestResults::Results(results) => {
                    let mut failure = false;
                    for (_i, result) in results.iter().enumerate() {
                        match result {
                            bfi::TestResult::Ok => {}
                            bfi::TestResult::RunTimeError(e) => {
                                println!("{:?}", e);
                                failure = true;
                            }
                            bfi::TestResult::UnexpectedOutput { expected, output } => {
                                assert_eq!(expected, output);
                            }
                        }
                    }
                    if failure {
                        panic!("One or more test failures");
                    }
                }
            };
        }
        Err(errors) => {
            for err in errors {
                println!("{}", err);
                panic!("Compilation failed");
            }
        }
    }
}

fn single_test(code: &str, input: Vec<u8>, output: Vec<u8>) {
    multiple_test(code, vec![input], vec![output])
}

// verify!("a b dup2", "a b a b");
fn fuzz(program: &str, returns: &str) {
    println!("Fuzzing: {}", program);

    // input order
    let program_inputs: Vec<_> = program
        .split_whitespace()
        .filter(|s| s.len() == 1 && s.chars().next().unwrap().is_ascii_lowercase())
        .map(|s| s.chars().next().unwrap())
        .collect();
    let program_outputs: Vec<_> = returns
        .split_whitespace()
        .filter(|s| s.len() == 1)
        .map(|s| s.chars().next().unwrap())
        .collect();

    let mut map = HashMap::new();
    for c in program_inputs.iter().chain(program_outputs.iter()) {
        map.insert(*c, 0);
    }

    // Generate many possible tests
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for _ in 0..100 {
        // Generate random inputs
        for c in program_inputs.iter() {
            if (*c).is_ascii_digit() {
                map.insert(*c, (*c).to_digit(10).unwrap() as u8);
            } else {
                map.insert(*c, rand::random());
            }
        }

        inputs.push(
            program_inputs
                .iter()
                .map(|c| map.get(c).unwrap())
                .cloned()
                .collect(),
        );

        outputs.push(
            program_outputs
                .iter()
                .map(|c| map.get(c).unwrap())
                .cloned()
                .rev()
                .collect(),
        );
    }

    // replace lowercase letters with "read"
    let code = program
        .split_whitespace()
        .map(|s| {
            if s.len() == 1 && s.chars().next().unwrap().is_ascii_lowercase() {
                "read".to_string()
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    multiple_test(
        &format!("IMPORT std; main == {code} {} popn;", program_outputs.len()),
        inputs,
        outputs,
    );
}

#[test]
fn ints() {
    // test creating all possible integers
    for i in 0u8..=255 {
        let code = &format!("IMPORT std; main == {} pop;", i);
        let input = vec![];
        let output = vec![i];
        single_test(code, input, output);
    }
}

#[test]
fn add() {
    let code = "IMPORT std; main == read read + pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // there are 256x256 = 65536 possible combinations of 2 numbers
    for i in 0u8..=255 {
        for j in 0u8..=255 {
            inputs.push(vec![i, j]);
            outputs.push(vec![i + j]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn sub() {
    let code = "IMPORT std; main == read read - pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // there are 256x256 = 65536 possible combinations of 2 numbers
    for i in 0u8..=255 {
        for j in 0u8..=255 {
            inputs.push(vec![i, j]);
            outputs.push(vec![i - j]);
        }
    }

    multiple_test(code, inputs, outputs);
}

// STACK MANIPULATION
#[test]
fn stack_manipulation() {
    fuzz("a", "a");
    // dup
    fuzz("a dup", "a a");
    fuzz("a b dup2", "a b a b");
    // drop
    fuzz("0 a drop", "0");
    fuzz("0 a b drop2", "0");
    // swap
    fuzz("a b swap", "b a");
    fuzz("a b c d swap2", "c d a b");
    // over
    fuzz("a b over", "a b a");
    fuzz("a b c d over2", "a b c d a b");
    // rot
    fuzz("a b c rot", "b c a");
    fuzz("a b c d e f rot2", "c d e f a b");
    // -rot
    fuzz("a b c -rot", "c a b");
    fuzz("a b c d e f -rot2", "e f a b c d");
    // nip
    fuzz("a b nip", "b");
    fuzz("a b c d nip2", "c d");
    // tuck
    fuzz("a b tuck", "b a b");
    fuzz("a b c d tuck2", "c d a b c d");
}

#[test]
fn inc() {
    let code = "IMPORT std; main == read inc pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test incrementing all numbers
    for i in 0u8..=255 {
        inputs.push(vec![i]);
        outputs.push(vec![i + 1]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn dec() {
    let code = "IMPORT std; main == read dec pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test decrementing all numbers
    for i in 0u8..=255 {
        inputs.push(vec![i]);
        outputs.push(vec![i - 1]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn eq() {
    let code = "IMPORT std; main == read read eq pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test comparing all numbers
    for i in 0u8..=255 {
        for j in 0u8..=255 {
            inputs.push(vec![i, j]);
            outputs.push(vec![if i == j { 1 } else { 0 }]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn zeq() {
    let code = "IMPORT std; main == read zeq pop;";
    let inputs = (0u8..=255).map(|i| vec![i]).collect();
    // [1] followed by 254 [0]
    let outputs = vec![vec![1]]
        .into_iter()
        .chain((0u8..=254).map(|_i| vec![0]))
        .collect();

    multiple_test(code, inputs, outputs);
}

#[test]
fn not() {
    let code = "IMPORT std; main == read not pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test inverting all numbers
    for i in 0u8..=255 {
        inputs.push(vec![i]);
        outputs.push(vec![if i == 0 { 1 } else { 0 }]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn neq() {
    let code = "IMPORT std; main == read read neq pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test comparing all numbers
    for i in 0u8..=255 {
        for j in 0u8..=255 {
            inputs.push(vec![i, j]);
            outputs.push(vec![if i != j { 1 } else { 0 }]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn dupn() {
    // `0 dupn` will behave like `drop drop`
    let code = "IMPORT std; main == read read dupn;";
    let inputs = (0u8..=255).map(|i| vec![i, 0]).collect();
    let outputs = vec![vec![]; 256];
    multiple_test(code, inputs, outputs);

    // `n dupn` will be checked for correctness by using n pops
    (1u8..=255).into_par_iter().for_each(|n| {
        let code = format!("IMPORT std; main == read read dupn {n} popn;");

        let inputs = (0..=10).map(|i| vec![i, n]).collect();
        let outputs = (0..=10).map(|i| vec![i; n as usize]).collect();

        multiple_test(&code, inputs, outputs);
    });
}

#[test]
fn popn() {
    single_test(
        "IMPORT std; main == 1 2 3 4 4 popn;",
        vec![],
        vec![4, 3, 2, 1],
    );
    single_test(
        "IMPORT std; main == 1 2 3 4 read popn;",
        vec![4],
        vec![4, 3, 2, 1],
    );
}

#[test]
fn mul() {
    let code = "IMPORT std; main == read read * pop;";
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test 100 random pairs
    for _ in 0..100 {
        let a = rand::random::<u8>();
        let b = rand::random::<u8>();

        inputs.push(vec![a, b]);
        outputs.push(vec![a * b]);
    }

    // add some special cases
    for i in 0u8..=255 {
        // 0 * anything = 0
        inputs.push(vec![0, i]);
        outputs.push(vec![0]);

        // anything * 0 = 0
        inputs.push(vec![i, 0]);
        outputs.push(vec![0]);

        // 1 * anything = anything
        inputs.push(vec![1, i]);
        outputs.push(vec![i]);

        // anything * 1 = anything
        inputs.push(vec![i, 1]);
        outputs.push(vec![i]);
    }

    multiple_test(code, inputs, outputs);
}
