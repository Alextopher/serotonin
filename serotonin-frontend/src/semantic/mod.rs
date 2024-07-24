use either::Either;
use errors::{SemanticError, SemanticWarning};
use lasso::{RodeoReader, Spur};
use symbol::SymbolTable;

use crate::ast::{Definition, Module};

// mod constraints;
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
