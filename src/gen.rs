use std::collections::HashMap;

use crate::{
    parse::{parser, AstNode, Ty},
    stdlib,
};

#[derive(Debug)]
pub enum TypeError {
    StackInsufficient { expected: i64, had: i64 },
    StackOverflow { limit: i64, had: i64 },
    UndefinedSymbol { name: String },
}

pub struct Gen {
    atomics: HashMap<String, (String, Ty)>,
}

impl Gen {
    pub fn builtins() -> Gen {
        let mut g = Gen {
            atomics: stdlib::builtin(),
        };
        g.load_lib();
        g
    }

    // more complex functions are written using the bultins
    fn load_lib(&mut self) {
        let root = parser(&stdlib::LIBRARY);
        self.gen(root);
    }

    fn test_quotation(
        &mut self,
        gift: i64,
        goal: Ty,
        quotation: &Vec<AstNode>,
    ) -> Option<TypeError> {
        let mut sz = gift + goal.pops;
        let mut last_sz;

        for node in quotation {
            match self.check_node(node) {
                Ok(Ty { pops, pushes }) => {
                    last_sz = sz;
                    sz += pushes - pops;
                    if sz < 0 {
                        return Some(TypeError::StackInsufficient {
                            expected: pops,
                            had: last_sz,
                        });
                    }
                }
                Err(e) => return Some(e),
            }
        }

        if goal.pushes - sz > 0 {
            Some(TypeError::StackInsufficient {
                expected: sz,
                had: goal.pops,
            })
        } else if goal.pushes - sz < 0 {
            Some(TypeError::StackOverflow {
                limit: goal.pushes,
                had: sz,
            })
        } else {
            None
        }
    }

    pub fn check_node(&mut self, prog: &AstNode) -> Result<Ty, TypeError> {
        match prog {
            AstNode::Compound(_, _, _) => unreachable!(),
            AstNode::Atomic(name) => match self.atomics.get(name) {
                Some((_, ty)) => Ok(*ty),
                None => return Err(TypeError::UndefinedSymbol { name: name.clone() }),
            },
            AstNode::Byte(_) => Ok(Ty { pushes: 1, pops: 0 }),
            AstNode::If { co, el, th } => {
                if let Some(e) = self.test_quotation(1, Ty { pops: 2, pushes: 1 }, co) {
                    return Err(e);
                }

                if let Some(e) = self.test_quotation(1, Ty { pops: 1, pushes: 1 }, th) {
                    return Err(e);
                }

                if let Some(e) = self.test_quotation(1, Ty { pops: 1, pushes: 1 }, el) {
                    return Err(e);
                }

                Ok(Ty { pops: 1, pushes: 0 })
            }
            AstNode::While { co, body } => {
                if let Some(e) = self.test_quotation(1, Ty { pops: 2, pushes: 1 }, co) {
                    return Err(e);
                }

                if let Some(e) = self.test_quotation(1, Ty { pops: 1, pushes: 1 }, body) {
                    return Err(e);
                }

                Ok(Ty { pops: 1, pushes: 1 })
            }
            AstNode::Definition(_name, ty, term) => {
                if let Some(e) = self.test_quotation(0, *ty, term) {
                    return Err(e);
                }

                Ok(*ty)
            }
        }
    }

    pub fn gen(&mut self, ast: AstNode) -> String {
        match ast {
            AstNode::Byte(byte) => number_speed(byte),
            AstNode::Atomic(name) => {
                if let Some(code) = self.atomics.get(&name) {
                    code.0.to_string()
                } else {
                    panic!("Undefined symbol: {}", name);
                }
            }
            AstNode::If { co, el, th } => {
                let co = self.quotation(co);
                let th = self.quotation(th);
                let el = self.quotation(el);
                format!("[->+>>+<<<]>[-<+>][-]+>[-]>{}[<<<[->>>>+<<<<]>>>>{}<<->>[<+>-]]<[>+<-]<[<[->>+<<]>>{}-]>>[-]<<<<", co, th, el)
            }
            AstNode::While { co, body } => {
                let cond = self.quotation(co);
                let body = self.quotation(body);
                format!(">[-]>[-]<<[->+>+<<]>>[-<<+>>]<{cond}[-[->>+<<]<{body}>[-]>[-]<<[->+>+<<]>>[-<<+>>]<{cond}][-]<")
            }
            AstNode::Definition(_, _, term) => self.quotation(term),
            AstNode::Compound(_name, private, public) => {
                // merge the private definitions into the definitions
                for definition in private.into_iter().chain(public.into_iter()) {
                    // compile the definition
                    let name = &definition.get_name();
                    let ty = self.check_node(&definition).unwrap();
                    let code = self.gen(definition);
                    self.atomics.insert(name.to_string(), (code, ty));
                }

                match self.atomics.get("main") {
                    Some((code, _)) => code.clone(),
                    None => String::new(),
                }
            }
        }
    }
    pub fn quotation(&mut self, quotation: Vec<AstNode>) -> String {
        quotation.into_iter().map(|node| self.gen(node)).collect()
    }
}

// generates the code to add a number to the stack
// TODO make a code-golfed solution for each number
fn number_speed<T>(n: T) -> String
where
    T: num::PrimInt + num::Unsigned,
{
    let mut result = String::from(">");

    num::range(T::zero(), n).for_each(|_| result.push_str("+"));

    result
}

// // using multiplication to generate numbers
// // it makes for smaller code but with how most optimizing compilers work
// // it is likely slower than using "+++++++..."
// fn number_lightly_golfed(n: u8) -> String {
//     String::from("")
// }

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

// // returns the two factors of n (a, b) such that a+b is minimized
// fn get_best_factor(n: u16) -> (u16, u16) {
//     let mut best_factor = (1, n);
//     let mut best_sum: u16 = n + 1;

//     for i in 2..=n {
//         if n % i == 0 {
//             let sum: u16 = i + n / i;
//             if sum < best_sum {
//                 best_factor = (i, n / i);
//                 best_sum = sum;
//             }
//         }
//     }

//     best_factor
// }
