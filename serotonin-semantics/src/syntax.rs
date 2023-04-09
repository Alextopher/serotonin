//! Preform semantic analysis on a Module.
//! - The same name and stack arguments can not be used twice.
//! - Verify that all functions are defined within scope
//! - Where applicable resolve function calls with fully qualified names

use std::collections::{HashMap, HashSet};

use codespan_reporting::diagnostic::Diagnostic;
use lasso::{Rodeo, Spur};

use serotonin_parser::{
    ast::{Definition, Module, StackArg, StackArgInner},
    TokenData,
};

#[derive(Debug, PartialEq, Eq)]
pub struct SemanticState<'a> {
    rodeo: &'a mut Rodeo,
    // Maps (module, function) to a list of mangled names that match that function call
    // Definitions are ordered by priority (last definition in a module is the highest priority)
    definitions: HashMap<(Spur, Spur), Vec<Spur>>,
    modules: HashMap<Spur, Module>,
}

impl<'a> SemanticState<'a> {
    pub fn new(rodeo: &'a mut Rodeo) -> Self {
        Self {
            rodeo,
            definitions: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn print_definitions(&self) {
        for ((module, function), mangled_names) in &self.definitions {
            println!(
                "{}.{}: {}",
                self.rodeo.resolve(module),
                self.rodeo.resolve(function),
                mangled_names
                    .iter()
                    .map(|mangled_name| self.rodeo.resolve(mangled_name))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
    }

    // 1st pass: Add all definitions to the state and check for duplicates
    pub fn add_module(&mut self, module: Module) -> Vec<Diagnostic<usize>> {
        let mut diagnostics = Vec::new();

        let mut buf = String::new();

        // Keep mangled names in a set to check for duplicates
        let mut mangled_names = HashSet::new();

        for definition in module.definitions() {
            let mangled_name = self.mangle_definition(definition, module.name(), &mut buf);

            if mangled_names.contains(&mangled_name) {
                diagnostics.push(
                    Diagnostic::error()
                        .with_message(format!(
                            "Duplicate definition: {}",
                            self.rodeo.resolve(&mangled_name)
                        ))
                        .with_labels(vec![
                            definition.span().primary_label("First definition here"),
                            definition.span().primary_label("Second definition here"),
                        ]),
                );
                continue;
            }

            let def = (module.name(), definition.name().spur());
            self.definitions.entry(def).or_default().push(mangled_name);

            mangled_names.insert(mangled_name);
        }

        self.modules.insert(module.name(), module);

        diagnostics
    }

    // Mangle the name of a definition to include the module name and stack arguments
    fn mangle_definition(&mut self, def: &Definition, module: Spur, buf: &mut String) -> Spur {
        buf.clear();

        buf.push_str(self.rodeo.resolve(&module));
        buf.push('.');
        buf.push_str(self.rodeo.resolve(&def.name().spur()));
        if let Some(stack) = &def.stack() {
            self.mangle_stack(stack.args(), buf);
        }

        self.rodeo.get_or_intern(buf)
    }

    // Mangle stack arguments into a common format
    fn mangle_stack(&mut self, stack: &[StackArg], buf: &mut String) {
        buf.push('(');

        // Map stack effects to a string
        let mut map = HashMap::new();

        let mut named_byte = 'a';
        let mut named_quotation = 'A';

        for arg in stack {
            match &arg.inner() {
                StackArgInner::UnnamedByte(_) => {
                    buf.push('@');
                }
                StackArgInner::UnnamedQuotation(_) => {
                    buf.push('?');
                }
                StackArgInner::NamedQuotation(token) => {
                    let c = token.spur();

                    // If the char has already been mapped to a name, use that name
                    if let Some(name) = map.get(&c) {
                        buf.push(*name);
                    } else {
                        // Otherwise, map the char to a name and use that name
                        map.insert(c, named_quotation);
                        buf.push(named_quotation);
                        named_quotation = (named_quotation as u8 + 1) as char;
                    }
                }
                StackArgInner::NamedByte(token) => {
                    let c = token.spur();

                    // If the char has already been mapped to a name, use that name
                    if let Some(name) = map.get(&c) {
                        buf.push(*name);
                    } else {
                        // Otherwise, map the char to a name and use that name
                        map.insert(c, named_byte);
                        buf.push(named_byte);
                        named_byte = (named_byte as u8 + 1) as char;
                    }
                }
                StackArgInner::Integer(token) => {
                    debug_assert!(matches!(token.data(), TokenData::Integer(_)));

                    let value = token.data().clone().unwrap_integer();
                    buf.push_str(&value.to_string());
                }
                StackArgInner::Quotation(_) => todo!("We can't compile quotation stack args yet"),
            }
        }

        buf.push(')');
    }
}
