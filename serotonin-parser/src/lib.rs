pub mod ast;
pub mod errors;
mod lexer;
mod parser;

use std::rc::Rc;

pub use lexer::InternedToken;
pub use lexer::Span;
pub use lexer::TokenData;
pub use lexer::TokenKind;

pub use lexer::lex;
pub use parser::{parse_definition, parse_module};

pub type Token = Rc<InternedToken>;