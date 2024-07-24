/// The lexer creates a stream of tokens from a file or string.
mod lexer;
/// The parser transforms a stream of tokens into an abstract syntax tree.
mod parser;
/// The semantic analyzer checks the AST for errors and creates a symbol table.
mod semantic;

pub use lexer::{lex, InternedToken, Span, Token, TokenData, TokenKind};
pub use parser::{ast, parse_definition, parse_module};
pub use semantic::SemanticAnalyzer;

pub(crate) const ICE_NOTE: &str =
    "This is a compiler error and should not have happened. Please report this as a bug.";
