use std::{rc::Rc, collections::HashMap};

use lasso::Spur;
use serotonin_parser::ast::{Definition, StackArg};

#[derive(Debug, PartialEq, Eq, Clone)]
enum StackItem {
    // a single byte
    Byte(u8),
    // A compiled brainfuck string 
    Quotation(Spur),
}

struct Name {
    name: Spur,
    definitions: Vec<Rc<Definition>>,
}

fn stack_matches_definition(stack: &[StackItem], definition: &Definition) -> bool {
    let mut items = stack.iter().rev();
    let Some(args) = definition.stack().map(|s| s.args().iter()) else {
        return false;
    };

    let mut state: HashMap<Spur, StackItem> = HashMap::new();

    for arg in args {
        let Some(next) = items.next() else {
            // There are less items to match than args in the definition 
            return false;
        };

        match (next, arg) {
            (StackItem::Byte(_), StackArg::UnnamedByte(_)) => {
                // skip
            },
            (StackItem::Byte(b), StackArg::NamedByte(t)) => {
                match state.get(&t.spur()) {
                    Some(item) => {
                        if *item == StackItem::Byte(*b) {
                            return false
                        }
                    },
                    None => {
                        state.insert(t.spur(), next.clone());
                    },
                }
            },
            (StackItem::Byte(b), StackArg::Integer(t)) => {
                if t.data().unwrap_byte() != *b {
                    return false
                }
            },
            (StackItem::Quotation(_), StackArg::UnnamedQuotation(_)) => {
                // skip
            },
            (StackItem::Quotation(q), StackArg::NamedQuotation(t)) => {
                match state.get(&t.spur()) {
                    Some(item) => {
                        if *item == StackItem::Quotation(*q) {
                            return false
                        }
                    },
                    None => {
                        // TODO: compile t
                        state.insert(t.spur(), next.clone());
                    },
                }
            },
            (StackItem::Quotation(q), StackArg::Quotation(t)) => {
                // TODO: Compile t and compare to q
                todo!()
            },
            _ => {
                return false
            }
        }
    }

    true
}

