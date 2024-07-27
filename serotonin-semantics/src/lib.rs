use either::Either;
use errors::{SemanticError, SemanticWarning};
use lasso::{RodeoReader, Spur};
use symbol::SymbolTable;

use serotonin_parser::ast::{Definition, Module};

mod errors;
mod solver;
mod symbol;

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
    rodeo: &'a RodeoReader,

    warnings: Vec<SemanticWarning>,
    errors: Vec<SemanticError>,

    symbol_table: SymbolTable<'a>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(rodeo: &'a RodeoReader) -> Self {
        Self {
            rodeo,
            errors: Vec::new(),
            warnings: Vec::new(),
            symbol_table: SymbolTable::new(rodeo),
        }
    }

    pub fn emit_warning(&mut self, warning: SemanticWarning) {
        self.warnings.push(warning);
    }

    pub fn emit_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }

    pub fn symbol_table(&self) -> &SymbolTable<'a> {
        &self.symbol_table
    }

    fn add_definition(
        &mut self,
        module: Spur,
        def: &'a Definition,
    ) -> Result<(), Either<SemanticError, SemanticWarning>> {
        todo!();
        // let constraints = match def.stack() {
        //     Some(stack) => self.stack_to_constraints(stack),
        //     None => Ok(Vec::new()),
        // }?;

        // self.symbol_table.insert(module, def, constraints);

        // Ok(())
    }

    pub fn analyze(&mut self, module: &'a Module) {
        let module_name = module.name();

        for def in module.definitions() {
            if let Err(e) = self.add_definition(module_name, def) {
                match e {
                    Either::Left(e) => self.emit_error(e),
                    Either::Right(w) => self.emit_warning(w),
                }
            }
        }
    }
}

/// Utility method that generates random (syntactically valid) BrainFuck programs.
#[cfg(test)]
pub(crate) fn random_brainfuck(n: usize) -> String {
    use std::cmp::Ordering;

    use rand::Rng;

    let mut rng = rand::thread_rng();
    let commands = ['+', '-', '>', '<', '.', ','];
    let mut program = String::with_capacity(n);

    // Track the number of unmatched '['
    let mut open_brackets = 0;

    for _ in 0..n {
        let cmd = if open_brackets == 0 {
            // Cannot insert a ']' if there are no unmatched '['
            let cmd_index = rng.gen_range(0..commands.len() + 1);
            if cmd_index < commands.len() {
                commands[cmd_index]
            } else {
                '['
            }
        } else {
            // Can insert any command, including '[' and ']'
            let cmd_index = rng.gen_range(0..commands.len() + 2);
            match cmd_index.cmp(&(commands.len())) {
                Ordering::Less => commands[cmd_index],
                Ordering::Equal => '[',
                Ordering::Greater => ']',
            }
        };

        match cmd {
            '[' => open_brackets += 1,
            ']' => {
                if open_brackets > 0 {
                    open_brackets -= 1;
                } else {
                    unreachable!()
                }
            }
            _ => {}
        }

        program.push(cmd);
    }

    // If there are unmatched '[' at the end, add the matching ']'
    for _ in 0..open_brackets {
        program.push(']');
    }

    program
}
