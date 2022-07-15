#![cfg(test)]
extern crate pest;

use crate::parser::BFJoyParser;
use rayon::prelude::*;
use std::num::Wrapping;

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

    // compile to brainfuck
    let code = parser.generate(ast.unwrap()).unwrap();
    assert!(!code.is_empty(), "code failed to compile");

    // run the code
    let errors = bf_instrumentator::run_bf_o3(&code, inputs, outputs);

    if !errors.is_empty() {
        errors.iter().for_each(|err| println!("{:?}", err));
    }
}

#[test]
fn ints() {
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
fn hex() {
    // test creating all possible hexadecimal numbers
    for i in (0u8..=255).map(Wrapping) {
        // main == N print;
        let code = format!("IMPORT std; main == {:#04x} pop;", i);
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
fn add() {
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
fn sub() {
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
fn dup() {
    let code = format!("IMPORT std; main == read dup pop pop;");
    let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
    let outputs = (0u8..=255).map(Wrapping).map(|i| vec![i, i]).collect();
    multiple_test(code, inputs, outputs);
}

#[test]
fn drop() {
    let code = format!("IMPORT std; main == read drop print;");
    let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
    let outputs = (0u8..=255)
        .map(Wrapping)
        .map(|_i| vec![Wrapping(0)])
        .collect();
    multiple_test(code, inputs, outputs);
}

#[test]
fn swap() {
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
fn over() {
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
fn rot() {
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
fn inc() {
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
fn dec() {
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
fn eq() {
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
fn zeq() {
    let code = format!("IMPORT std; main == read zeq pop;");
    let inputs = (0u8..=255).map(|i| vec![Wrapping(i)]).collect();
    // [1] followed by 254 [0]
    let outputs = vec![vec![Wrapping(1)]]
        .into_iter()
        .chain((0u8..=254).map(|_i| vec![Wrapping(0)]))
        .collect();

    multiple_test(code, inputs, outputs);
}

#[test]
fn not() {
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
fn neq() {
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
// fn ifte() {
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

#[test]
fn dupn() {
    // `0 dupn` will behave like `drop drop`
    let code = format!("IMPORT std; main == read 0 dupn;");
    let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
    let outputs = vec![];
    multiple_test(code, inputs, outputs);

    // `n dupn` will be checked for correctness by using n pops
    (1..=255).into_par_iter().for_each(|n| {
        let pops = (0..n).map(|_| "pop").collect::<Vec<_>>().join(" ");
        let code = format!("IMPORT std; main == read {n} dupn {pops};");

        let inputs = (0..=10).map(|i| vec![Wrapping(i)]).collect();
        let outputs = (0..=10).map(|i| vec![Wrapping(i); n]).collect();

        multiple_test(code, inputs, outputs);
    })
}

#[test]
fn mul() {
    let code = format!("IMPORT std; main == read read mul pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test 100 random pairs
    for _ in 0..100 {
        let a = rand::random::<Wrapping<u8>>();
        let b = rand::random::<Wrapping<u8>>();

        inputs.push(vec![a, b]);
        outputs.push(vec![a * b]);
    }

    // add some special cases
    for i in (0u8..=255).map(Wrapping) {
        // 0 * anything = 0
        inputs.push(vec![Wrapping(0), i]);
        outputs.push(vec![Wrapping(0)]);

        // anything * 0 = 0
        inputs.push(vec![i, Wrapping(0)]);
        outputs.push(vec![Wrapping(0)]);

        // 1 * anything = anything
        inputs.push(vec![Wrapping(1), i]);
        outputs.push(vec![i]);

        // anything * 1 = anything
        inputs.push(vec![i, Wrapping(1)]);
        outputs.push(vec![i]);
    }

    multiple_test(code, inputs, outputs);
}

// Tests for the u16 module
#[test]
fn adc() {
    let code = format!("IMPORT std u16; main == read read addc pop pop;");
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

#[test]
fn incc() {
    let code = format!("IMPORT std u16; main == read incc pop pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for i in 0u16..=255 {
        inputs.push(vec![Wrapping(i as u8)]);
        if i == 255 {
            outputs.push(vec![Wrapping(1), Wrapping(0)]);
        } else {
            outputs.push(vec![Wrapping(0), Wrapping(i as u8 + 1)]);
        }
    }

    multiple_test(code, inputs, outputs);
}
