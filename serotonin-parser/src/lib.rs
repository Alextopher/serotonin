use std::rc::Rc;

pub mod ast;
pub mod errors;
mod lexer;
mod parser;

pub type TokenType = Rc<InternedToken>;

pub use lexer::InternedToken;
pub use lexer::Span;
pub use lexer::Token;
pub use lexer::TokenData;

pub use lexer::lex;
pub use parser::{parse_definition, parse_module};
