//! Serotonin is a fairly unique language in that our definitions almost always have multiple definitions for optimization purposes.
//!
//! Consider dupn:
//!
//! ```sero
//! dupn == [] [dec over swap] while drop;
//! dupn (n) ==? 0 '[-' n [] [dec `[->>+<<]<` '>+' `>`] while '<' n dupn ']>[>]<[-' '<' n dupn '+' '>' n dupn ']' sprint;
//! dupn (0) == drop drop;
//! dupn (a n) ==! n [] [dec `[->+<]<` a `>`] while sprint;
//! dupn (0 n) ==! n [] [dec `>.<`] while;
//! ```
//!
//! To give some more context for the language. When you use a specific definition it is called 'applying' the definition. Definitions have
//! arguments in the form of 'constraints'. `(0 n)` is a constraint the second stack element is exactly the byte `0` and the top of the stack is
//! any specific value we call `n`. `0 10 dupn` would apply the definition `dupn (0 n)` with `n` being `10`.
//!
//! In order to resolve which definition to apply serotonin looks from bottom-to-top in the file for the first definition that is applicable.
//! You should think of the order of definitions as going from "most general" to "most specific". If a more general definition is written
//! after a more specific definition, the more general one will end up being unreachable. The semantic analyzer checks for this and emits a warning.

use std::collections::HashMap;

use lasso::{RodeoReader, Spur};

use crate::ast::Definition;

use super::solver::Constraint;

/// Symbol table for a single module
type ModuleTable<'a> = HashMap<Spur, Vec<(&'a Definition, Constraint)>>;

/// Symbol table for the semantic analyzer
///
/// The symbol table is a map from a symbol to a list of definitions.
///
/// Definitions are ordered in increasing priority, so most usages require reverse iteration.
#[derive(Debug, PartialEq, Eq)]
pub struct SymbolTable<'a> {
    rodeo: &'a RodeoReader,
    symbols: HashMap<Spur, ModuleTable<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(rodeo: &'a RodeoReader) -> Self {
        Self {
            rodeo,
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, module: Spur, definition: &'a Definition, constraint: Constraint) {
        self.symbols
            .entry(module)
            .or_default()
            .entry(definition.name().spur())
            .or_default()
            .push((definition, constraint));
    }
}

impl std::fmt::Display for SymbolTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (module, table) in &self.symbols {
            writeln!(f, "Module: {}", self.rodeo.resolve(module))?;

            for definitions in table.values() {
                for (definition, constraints) in definitions {
                    // write the definition name on 1 line, then a list of constraints on the next
                    write!(f, "  {}", self.rodeo.resolve(&definition.name().spur()))?;

                    if !constraints.is_empty() {
                        writeln!(f, ":")?;
                        write!(f, "    ")?;
                        for constraint in constraints.iter() {
                            write!(f, "{:?} ", constraint)?;
                        }
                    }
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}
