mod errors;
mod lex;
mod span;
mod token;

pub use errors::TokenizerError;
pub use lex::lex;
pub use span::Span;
pub use token::InternedToken;
pub use token::TokenData;
pub use token::TokenKind;


#[derive(Debug, Clone)]
pub struct Token {
    pub(crate) tokens: &'static [InternedToken],
    pub(crate) index: usize,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.tokens[self.index] == other.tokens[other.index]
    }
}

impl Eq for Token {}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tokens[self.index].hash(state);
    }
}

impl std::ops::Deref for Token {
    type Target = InternedToken;

    fn deref(&self) -> &Self::Target {
        &self.tokens[self.index]
    }
}