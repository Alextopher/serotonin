pub mod ast;

/// The lexer creates a stream of tokens from a file or string.
mod lexer;
/// The parser transforms a stream of tokens into an abstract syntax tree.
mod parser;

pub use lexer::{InternedToken, Span, Token, TokenData, TokenKind, lex};
pub use parser::{parse_definition, parse_module};
