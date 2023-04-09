use std::{rc::Rc, collections::HashMap};

use lasso::Spur;
use serotonin_parser::ast::{Definition, StackArg};

enum StackItem {
    Byte(u8),
    Quotation(String), // brainfuck
}

struct Name {
    name: Spur,
    definitions: Vec<Rc<Definition>>,
}

struct State {
    
}

fn matches(definition: &Definition, stack: &[StackItem]) -> bool {
    let mut items = stack.iter().rev();
    let Some(mut args) = definition.stack().map(|s| s.args().iter()) else {
        return false;
    };

    let mut state: HashMap<Spur, StackItem> = HashMap::new();

    for arg in args {
        match arg {
            StackArg::UnnamedByte(_) => todo!(),
            StackArg::UnnamedQuotation(_) => todo!(),
            StackArg::NamedByte(_) => todo!(),
            StackArg::NamedQuotation(_) => todo!(),
            StackArg::Integer(_) => todo!(),
            StackArg::Quotation(_) => todo!(),
        }
    }

    false
}

