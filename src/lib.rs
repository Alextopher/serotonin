extern crate rand;
extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod parse;
pub mod stdlib;
pub mod gen;

#[cfg(test)]
mod tests {
    extern crate pest;

    use std::num::Wrapping;

    use crate::{stdlib, parse, gen};

    // a dead simple BF interpreter
    fn run(bf: &str, input: Vec<Wrapping<u8>>) -> Vec<Wrapping<u8>> {
        // tape is maxiumum of 2^16 cells
        let mut tape = [Wrapping(0u8); 65536];
        let mut tape_ptr = 0;
        let mut inst_ptr = 0;

        let mut input_ptr = 0;
        let mut output = Vec::new();

        // run the BF code
        while inst_ptr < bf.len() {
            let c = bf.as_bytes()[inst_ptr];
            match c {
                b'>' => tape_ptr += 1,
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
                },  
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

        output
    }

    fn simple_test(joy: String, input: Vec<Wrapping<u8>>, output: Vec<Wrapping<u8>>) {
        // add built-in functions
        let mut compiled = stdlib::builtin();

        // compile the standard library
        stdlib::load_lib(&mut compiled);

        // build the AST
        let definition = parse::parse_single_definition(&joy);

        // compile
        let code = gen::gen_bf(definition, &compiled);

        // run the code
        let result = run(code.as_str(), input);

        assert_eq!(result, output, "Expected output {:?} but got {:?}", output, result);
    }

    fn multiple_test(joy: String, inputs: Vec<Vec<Wrapping<u8>>>, outputs: Vec<Vec<Wrapping<u8>>>) {
        // add built-in functions
        let mut compiled = stdlib::builtin();

        // compile the standard library
        stdlib::load_lib(&mut compiled);

        // build the AST
        let definition = parse::parse_single_definition(&joy);

        // compile
        let code = gen::gen_bf(definition, &compiled);
        let s = code.as_str();

        // run the code
        for (input, output) in inputs.iter().zip(outputs.iter()) {
            let result = run(&s, input.clone());
            assert_eq!(result, output.clone(), "Expected output {:?} but got {:?}", output, result);
        }
    }

    #[test]
    fn test_ints() {
        // test creating all possible integers
        for i in (0u8..=255).map(Wrapping) {
            // main 0:1 == N print;
            let code = format!("main 0:1 == {} print", i);
            let input = vec![];
            let output = vec![i];
            simple_test(code, input, output);
        }
    }

    #[test]
    fn test_hex() {
        // test creating all possible hexadecimal numbers
        for i in (0u8..=255).map(Wrapping) {
            // main 0:1 == N print;
            let code = format!("main 0:1 == {:#04x} print", i);
            println!("{}", code);
            let input = vec![];
            let output = vec![i];
            simple_test(code, input, output);
        }
    }

    #[test]
    fn text_read_pop() {
        let code = format!{"main 0:0 == read read - pop"};
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
    fn test_add() {
        let code = format!{"main 0:0 == read read + pop"};
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
        // there are 256x256 = 65536 possible combinations of 2 numbers
        // I will test all of them (both ways)
        for i in (0u8..=255).map(Wrapping) {
            for j in (0u8..=255).map(Wrapping) {
                // main 0:1 == N print;
                let code = format!("main 0:1 == {} {} - print", i, j);
                let input = vec![];
                let output = vec![i - j];
                simple_test(code, input, output);
            }
        }
    }

    #[test]
    fn test_dup() {
        let code = format!("main 0:0 == read dup pop pop");
        let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
        let outputs = (0u8..=255).map(Wrapping).map(|i| vec![i, i]).collect();
        multiple_test(code, inputs, outputs);
    }
    
    #[test]
    fn test_drop() {
        let code = format!("main 0:0 == read drop print");
        let inputs = (0u8..=255).map(Wrapping).map(|i| vec![i]).collect();
        let outputs = (0u8..=255).map(Wrapping).map(|_i| vec![Wrapping(0)]).collect();
        multiple_test(code, inputs, outputs);
    }

    #[test]
    fn test_swap() {
        // main 0:0 == read read swap pop pop;
        let code = format!("main 0:0 == read read swap pop pop");
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
        // main 0:0 == read read over pop pop pop;
        let code = format!("main 0:0 == read read over pop pop pop");
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
        let code = format!("main 0:0 == read read read rot pop pop pop");
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
        let code = format!("main 0:0 == read inc pop");
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
        let code = format!("main 0:0 == read dec pop");
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // test decrementing all numbers
        for i in (0u8..=255).map(Wrapping) {
            inputs.push(vec![i]);
            outputs.push(vec![i - Wrapping(1)]);
        }

        multiple_test(code, inputs, outputs);
    }
}