#![cfg(test)]

extern crate pest;

use std::{cmp::max, num::Wrapping, process::exit};

use crate::parser::BFJoyParser;

// a dead simple BF interpreter
fn run(bf: &str, input: Vec<Wrapping<u8>>) -> (Vec<Wrapping<u8>>, [Wrapping<u8>; 65536], usize) {
    // tape is maxiumum of 2^16 cells
    let mut tape = [Wrapping(0u8); 65536];
    let mut tape_ptr = 0;
    let mut max_tape_ptr = 0;
    let mut inst_ptr = 0;

    let mut input_ptr = 0;
    let mut output = Vec::new();

    // run the BF code
    while inst_ptr < bf.len() {
        let c = bf.as_bytes()[inst_ptr];
        match c {
            b'>' => {
                tape_ptr += 1;
                max_tape_ptr = max(max_tape_ptr, tape_ptr);
            }
            b'<' => tape_ptr -= 1,
            b'+' => tape[tape_ptr] += 1,
            b'-' => tape[tape_ptr] -= 1,
            b'.' => {
                output.push(tape[tape_ptr]);
            }
            b',' => {
                if input_ptr < input.len() {
                    tape[tape_ptr] = input[input_ptr];
                    input_ptr += 1;
                } else {
                    panic!("Input ended before BF code");
                }
            }
            b'[' => {
                if tape[tape_ptr] == Wrapping(0) {
                    // skip forward to the matching ]
                    let mut depth = 1;
                    while depth > 0 {
                        inst_ptr += 1;
                        if bf.as_bytes()[inst_ptr] == b'[' {
                            depth += 1;
                        } else if bf.as_bytes()[inst_ptr] == b']' {
                            depth -= 1;
                        }
                    }
                }
            }
            b']' => {
                if tape[tape_ptr] != Wrapping(0) {
                    // skip backward to the matching [
                    let mut depth = 1;
                    while depth > 0 {
                        inst_ptr -= 1;
                        if bf.as_bytes()[inst_ptr] == b']' {
                            depth += 1;
                        } else if bf.as_bytes()[inst_ptr] == b'[' {
                            depth -= 1;
                        }
                    }
                }
            }
            _ => {}
        }
        inst_ptr += 1;
    }

    (output, tape, max_tape_ptr)
}

fn verify_tape(tape: [Wrapping<u8>; 65536], max_tape_ptr: usize) -> Option<usize> {
    // verifies the tape is set to zeros
    for i in 0..max_tape_ptr {
        if tape[i] != Wrapping(0) {
            return Some(i);
        }
    }

    None
}

fn single_test(joy: String, input: Vec<Wrapping<u8>>, output: Vec<Wrapping<u8>>) {
    multiple_test(joy, vec![input], vec![output])
}

fn multiple_test(joy: String, inputs: Vec<Vec<Wrapping<u8>>>, outputs: Vec<Vec<Wrapping<u8>>>) {
    // build the AST
    let mut parser = BFJoyParser::new();

    let ast = parser.module(&joy, "test".to_string());

    if let Err(err) = ast {
        panic!("Failed to parse file {}", err);
    }

    let code = parser.generate(ast.unwrap()).unwrap();

    // compile
    assert!(!code.is_empty(), "code failed to compile");

    // run the code
    for (input, output) in inputs.iter().zip(outputs.iter()) {
        let (result, tape, max_tape) = run(&code, input.clone());

        // verify the tape is set to zeros
        match verify_tape(tape, max_tape) {
            Some(i) => panic!("Input {:?}: Tape is {} at {}", input, tape[i], i),
            None => {}
        }

        assert_eq!(
            result,
            output.clone(),
            "Input {:?} expected output {:?} but got {:?}",
            input,
            output,
            result
        );
    }
}

#[test]
fn test_ints() {
    // test creating all possible integers
    for i in (0u8..=255).map(Wrapping) {
        // main == N print;
        let code = format!("IMPORT std; main == {} pop;", i);
        let input = vec![];
        let output = vec![i];
        single_test(code, input, output);
    }
}

#[test]
fn test_hex() {
    // test creating all possible hexadecimal numbers
    for i in (0u8..=255).map(Wrapping) {
        // main == N print;
        let code = format!("IMPORT std; main == {:#04x} pop;", i);
        println!("{}", code);
        let input = vec![];
        let output = vec![i];
        single_test(code, input, output);
    }
}

#[test]
fn text_read_pop() {
    let code = format!("IMPORT std; main == read read pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // there are 256x256 = 65536 possible combinations of 2 numbers
    for i in (0u8..=255).map(Wrapping) {
        for j in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i, j]);
            outputs.push(vec![j, i]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_add() {
    let code = format!("IMPORT std; main == read read + pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // there are 256x256 = 65536 possible combinations of 2 numbers
    for i in (0u8..=255).map(Wrapping) {
        for j in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i, j]);
            outputs.push(vec![i + j]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_sub() {
    let code = format!("IMPORT std; main == read read - pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // there are 256x256 = 65536 possible combinations of 2 numbers
    for i in (0u8..=255).map(Wrapping) {
        for j in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i, j]);
            outputs.push(vec![i - j]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_dup() {
    let code = format!("IMPORT std; main == read dup pop pop;");
    let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
    let outputs = (0u8..=255).map(Wrapping).map(|i| vec![i, i]).collect();
    multiple_test(code, inputs, outputs);
}

#[test]
fn test_drop() {
    let code = format!("IMPORT std; main == read drop print;");
    let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
    let outputs = (0u8..=255)
        .map(Wrapping)
        .map(|_i| vec![Wrapping(0)])
        .collect();
    multiple_test(code, inputs, outputs);
}

#[test]
fn test_swap() {
    // main == read read swap pop pop;
    let code = format!("IMPORT std; main == read read swap pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test swapping some random numbers
    (0..100).for_each(|_i| {
        let a = rand::random::<Wrapping<u8>>();
        let b = rand::random::<Wrapping<u8>>();

        inputs.push(vec![a, b]);
        outputs.push(vec![a, b]);
    });

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_over() {
    // main == read read over pop pop pop;
    let code = format!("IMPORT std; main == read read over pop pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test swapping some random numbers
    (0..100).for_each(|_i| {
        let a = rand::random::<Wrapping<u8>>();
        let b = rand::random::<Wrapping<u8>>();

        inputs.push(vec![a, b]);
        outputs.push(vec![a, b, a]);
    });

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_rot() {
    let code = format!("IMPORT std; main == read read read rot pop pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test rotating some random numbers
    (0..100).for_each(|_i| {
        let a = rand::random::<Wrapping<u8>>();
        let b = rand::random::<Wrapping<u8>>();
        let c = rand::random::<Wrapping<u8>>();

        inputs.push(vec![a, b, c]);
        outputs.push(vec![a, c, b]);
    });

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_inc() {
    let code = format!("IMPORT std; main == read inc pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test incrementing all numbers
    for i in (0u8..=255).map(Wrapping) {
        inputs.push(vec![i]);
        outputs.push(vec![i + Wrapping(1)]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_dec() {
    let code = format!("IMPORT std; main == read dec pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test decrementing all numbers
    for i in (0u8..=255).map(Wrapping) {
        inputs.push(vec![i]);
        outputs.push(vec![i - Wrapping(1)]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_eq() {
    let code = format!("IMPORT std; main == read read eq pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test comparing all numbers
    for i in (0u8..=255).map(Wrapping) {
        for j in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i, j]);
            outputs.push(vec![if i == j { Wrapping(1) } else { Wrapping(0) }]);
        }
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_not() {
    let code = format!("IMPORT std; main == read not pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test inverting all numbers
    for i in (0u8..=255).map(Wrapping) {
        inputs.push(vec![i]);
        outputs.push(vec![if i == Wrapping(0) {
            Wrapping(1)
        } else {
            Wrapping(0)
        }]);
    }

    multiple_test(code, inputs, outputs);
}

#[test]
fn test_neq() {
    let code = format!("IMPORT std; main == read read neq pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test comparing all numbers
    for i in (0u8..=255).map(Wrapping) {
        for j in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i, j]);
            outputs.push(vec![if i != j { Wrapping(1) } else { Wrapping(0) }]);
        }
    }

    multiple_test(code, inputs, outputs);
}

// #[test]
// fn test_ifte() {
//     let code = format!("IMPORT std; main == read [read eq] [inc pop] [dec pop] ifte;");
//     let mut inputs = Vec::new();
//     let mut outputs = Vec::new();

//     // test many comparisons
//     for i in (0u8..=255).map(Wrapping) {
//         for j in (0u8..=255).map(Wrapping) {
//             inputs.push(vec![i, j]);
//             outputs.push(vec![if i == j {
//                 i + Wrapping(1)
//             } else {
//                 i - Wrapping(1)
//             }]);
//         }
//     }

//     multiple_test(code, inputs, outputs);
// }

// #[test]
// fn test_while() {
//     let code = format!("IMPORT std; main == 0 pop 1 [0 neq] [print dup +] while drop;");
//     let input = vec![];
//     let output = vec![
//         Wrapping(0),
//         Wrapping(1),
//         Wrapping(2),
//         Wrapping(4),
//         Wrapping(8),
//         Wrapping(16),
//         Wrapping(32),
//         Wrapping(64),
//         Wrapping(128),
//     ];

//     single_test(code, input, output);
// }

#[test]
// verifies if add with carry works correctly on all inputs
fn test_adc() {
    let code = format!("IMPORT std u16; main == read read adc pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for i in 0u16..=255 {
        for j in 0u16..=255 {
            inputs.push(vec![Wrapping(i as u8), Wrapping(j as u8)]);

            let sum = i + j;
            if sum > 255 {
                outputs.push(vec![Wrapping(1), Wrapping(i as u8) + Wrapping(j as u8)]);
            } else {
                outputs.push(vec![Wrapping(0), Wrapping(i as u8) + Wrapping(j as u8)]);
            }
        }
    }

    multiple_test(code, inputs, outputs);
}