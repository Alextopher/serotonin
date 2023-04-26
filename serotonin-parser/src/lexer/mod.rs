mod errors;
mod lex;
mod span;
mod token;

use std::rc::Rc;

pub use errors::TokenizerError;
pub use lex::lex;
pub use span::Span;
pub use token::InternedToken;
pub use token::TokenData;
pub use token::TokenKind;

pub type Token = Rc<InternedToken>;
