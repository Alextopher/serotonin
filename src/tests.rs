#![cfg(test)]
extern crate pest;

use crate::parser::BFJoyParser;
use std::num::Wrapping;

#[derive(Debug, PartialEq, Clone)]
enum Op {
    // add or subtract
    Change(i32),
    // move left or right
    Move(i32),
    // .
    Print,
    // ,
    Read,
    // [ ... ]
    Loop(Box<[Op]>),
    // [-]
    Zero,
}

fn parse(it: &mut impl Iterator<Item = u8>) -> Box<[Op]> {
    let mut buf = vec![];
    while let Some(c) = it.next() {
        buf.push(match c {
            b'-' => Op::Change(-1),
            b'+' => Op::Change(1),
            b'<' => Op::Move(-1),
            b'>' => Op::Move(1),
            b'.' => Op::Print,
            b',' => Op::Read,
            b'[' => Op::Loop(parse(it)),
            b']' => break,
            _ => continue,
        });
    }
    buf.into_boxed_slice()
}

fn optimize(ops: Box<[Op]>) -> Box<[Op]> {
    fn replace_zeros(ops: Box<[Op]>) -> Box<[Op]> {
        // check if the loop is [-] or [+]
        if ops.len() == 1 && (ops[0] == Op::Change(1) || ops[0] == Op::Change(-1)) {
            Box::new([Op::Zero])
        } else {
            let mut new_ops = vec![];
            for op in ops.iter() {
                let new_op = op.clone();
                match new_op {
                    Op::Loop(ops) => new_ops.push(Op::Loop(replace_zeros(ops.clone()))),
                    _ => new_ops.push(new_op),
                }
            }
            new_ops.into_boxed_slice()
        }
    }

    let ops = replace_zeros(ops);
    let mut new_ops: Vec<Op> = vec![];

    for op in ops.into_iter() {
        if new_ops.len() == 0 {
            new_ops.push(op.clone());
        } else {
            match (new_ops.last_mut().unwrap(), op) {
                (Op::Change(a), Op::Change(b)) => {
                    *a += b;
                }
                (Op::Move(a), Op::Move(b)) => {
                    *a += b;
                }
                (Op::Change(_), Op::Zero) => {
                    new_ops.pop();
                    new_ops.push(Op::Zero);
                }
                (Op::Zero, Op::Loop(_)) => {}
                (Op::Loop(_), Op::Loop(_)) => {}
                (_, op) => new_ops.push(op.clone()),
            }
        }
    }

    new_ops.into_boxed_slice()
}

// a dead simple BF interpreter
struct Interpreter {
    tape: [Wrapping<u8>; 65536],
    pointer: usize,
    input_pointer: usize,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            tape: [Wrapping(0); 65536],
            pointer: 0,
            input_pointer: 0,
        }
    }

    fn run(
        &mut self,
        instructions: Box<[Op]>,
        input: &Vec<Wrapping<u8>>,
        output: &mut Vec<Wrapping<u8>>,
    ) {
        for instruction in instructions.iter() {
            match instruction {
                Op::Change(change) => {
                    if *change > 0 {
                        self.tape[self.pointer] += Wrapping(*change as u8);
                    } else {
                        self.tape[self.pointer] -= Wrapping(-change as u8);
                    }
                }
                Op::Move(move_) => {
                    if *move_ > 0 {
                        self.pointer += *move_ as usize;
                    } else {
                        self.pointer -= (-move_) as usize;
                    }
                }
                Op::Print => output.push(self.tape[self.pointer]),
                Op::Read => {
                    if self.input_pointer < input.len() {
                        self.tape[self.pointer] = input[self.input_pointer];
                        self.input_pointer += 1;
                    } else {
                        panic!("not enough input");
                    }
                }
                Op::Loop(instructions) => {
                    while self.tape[self.pointer] != Wrapping(0) {
                        self.run(instructions.clone(), input, output);
                    }
                }
                Op::Zero => self.tape[self.pointer] = Wrapping(0),
            }
        }
    }
}

fn verify_tape(tape: [Wrapping<u8>; 65536]) -> Option<usize> {
    // verifies the tape is set to zeros
    for i in 0..65536 {
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

    // optimize the brainfuck code
    let instructions = optimize(parse(&mut code.bytes()));
    println!("{:?}", instructions);

    // run the code
    for (input, output) in inputs.iter().zip(outputs.iter()) {
        // run the interpreter
        let mut result = vec![];
        let mut interpreter = Interpreter::new();
        interpreter.run(instructions.clone(), input, &mut result);

        // verify the tape is set to zeros
        match verify_tape(interpreter.tape) {
            Some(i) => panic!(
                "Input {:?}: NonZeroTape at position {}\n{:?}",
                input, i, interpreter.tape
            ),
            None => {}
        }

        assert_eq!(
            result,
            output.clone(),
            "Input {:?}: expected output {:?} but got {:?}",
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
fn test_zeq() {
    let code = format!("IMPORT std; main == read zeq pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

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

#[test]
fn test_mul() {
    // for now this just tests multiply because that's the only thing that uses dupn
    let code = format!("IMPORT std; main == read read mul pop;");
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // test 1000 random pairs
    (0..100).for_each(|_i| {
        let a = rand::random::<Wrapping<u8>>();
        let b = rand::random::<Wrapping<u8>>();

        inputs.push(vec![a, b]);
        outputs.push(vec![a * b]);
    });

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
fn test_adc() {
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
fn test_incc() {
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
