use std::collections::HashMap;

use num::abs;

use crate::parse::AstNode;

fn gen(ast: AstNode, compiled: &HashMap<String, String>) -> String {
    let mut stack: Vec<String> = Vec::new();

    match ast {
        AstNode::Byte(byte) => {
            stack.push(number_speed(byte));
        }
        AstNode::Atomic(name) => {
            if let Some(code) = compiled.get(&name) {
                stack.push(code.to_string());
            } else {
                panic!("Undefined symbol: {}", name);
            }
        }
        AstNode::Term(factors) => {
            for factor in factors {
                // Check if factor is atomic and "if"
                if let AstNode::Atomic(name) = &factor {
                    if name == "ifte" {    
                        let el = match stack.pop() {
                            Some(el) => el,
                            None => panic!("ifte syntax error: [condition] [then] [else] if"),
                        };
       
                        let th = match stack.pop() {
                            Some(th) => th,
                            None => panic!("ifte syntax error: [condition] [then] [else] if"),
                        };

                        let co: String = match stack.pop() {
                            Some(co) => co,
                            None => panic!("ifte syntax error: [condition] [then] [else] if"),
                        };

                        // please don't ask me how
                        stack.push(format!("[->+>>+<<<]>[-<+>][-]+>[-]>{}[<<<[->>>>+<<<<]>>>>{}<<->>[<+>-]]<[>+<-]<[<[->>+<<]>>{}-]>>[-]<<<<", co, th, el));
                    } else if name == "while" {
                        let code = match stack.pop() {
                            Some(code) => code,
                            None => panic!("while syntax error: [condition] [code] while"),
                        };

                        let cond: String = match stack.pop() {
                            Some(cond) => cond,
                            None => panic!("while syntax error: [condition] [code] while"),
                        };

                        // {condition}[{code}{condition}]
                        stack.push(format!(">[-]>[-]<<[->+>+<<]>>[-<<+>>]<{0}[-[->>+<<]<{1}>[-]>[-]<<[->+>+<<]>>[-<<+>>]<{0}][-]<", cond, code));
                    } else {
                        stack.push(gen(factor, compiled));
                    }
                } else {
                    stack.push(gen(factor, compiled));
                }
            }
        }
        _ => unreachable!(),
    }

    let mut result = String::new();
    // print the stack in reverse order
    for code in stack.iter() {
        result.push_str(&code);
    }
    result
}

// Converts a definition to brainfuck
pub fn gen_bf(definition: Box<AstNode>, compiled: &HashMap<String, String>) -> String {
    if let AstNode::Definition(_, _, _, term) = *definition {
        gen(*term, compiled)
    } else {
        panic!("Not a definition");
    }
}

// generates the code to add a number to the stack
// TODO make a code-golfed solution for each number
fn number_speed<T>(n: T) -> String
where
    T: num::PrimInt + num::Unsigned
{
    let mut result = String::from(">");

    num::range(T::zero(), n)
        .for_each(|_| result.push_str("+"));

    result
}

// using multiplication to generate numbers
// it makes for smaller code but with how most optimizing compilers work
// it is likely slower than using "+++++++..."
fn number_lightly_golfed(n: u8) -> String {
    String::from("")
}

// pub fn generate_constants() -> Vec<String> {
//     // 256 different constants
//     let mut constants: Vec<String> = Vec::new();

//     for i in 0..=255 {
//         // firstly generate the basic code
//         // >{+}a[<{+}b>-]
//         let (a, b) = get_best_factor(i);

//         if a == 1 {
//             constants.push(number_speed(i));   
//         } else {
//             let mut code = String::from(">");
//             for _ in 0..a {
//                 code.push('+');
//             }
//             code.push_str("[<");
//             for _ in 0..b {
//                 code.push('+');
//             }
//             code.push_str(">-]");    
//             constants.push(code);
//         }
//     }

//     // Now we preform some optimizations
//     let mut changed = false;
//     while changed {
//         for i in 0..constants.len() {
//             // look up and down 10 spots
//             for j in i-10..=i+10 {
//                 if i == j {
//                     continue
//                 }

//                 // compare the length of j to i
//                 if constants[i].len() < constants[j].len() + abs(j) {
                    
//                 }
//             }
//         }
//     }


//     constants
// }

// returns the two factors of n (a, b) such that a+b is minimized
fn get_best_factor(n: u16) -> (u16, u16) {
    let mut best_factor = (1, n);
    let mut best_sum: u16 = n + 1;

    for i in 2..=n {
        if n % i == 0 {
            let sum: u16 = i + n / i;
            if sum < best_sum {
                best_factor = (i, n / i);
                best_sum = sum;
            }
        }
    }

    best_factor
}