use lasso::Spur;

use crate::ast::Module;

use super::{errors::ParseError, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_module(&mut self, name: Spur) -> Result<Module, ParseError> {
        self.skip_trivia();
        let imports = self.optional_imports();
        let imports = match imports {
            Some(i) => Some(i?),
            None => None,
        };

        // While we keep finding tokens, parse definitions
        let mut definitions = Vec::new();
        loop {
            // skip trivia
            self.skip_trivia();
            if self.peek().is_none() {
                break;
            }
            definitions.push(self.parse_definition()?);
        }

        Ok(Module::new(name, imports, definitions))
    }
}
