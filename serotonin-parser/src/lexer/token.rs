use lasso::{Rodeo, Spur};
use logos::Logos;

use crate::Span;

/// A token that has been interned and has a span.
/// 
/// Note: InternedToken does not implement `Clone` because it is not intended to be cloned.
#[derive(Debug)]
pub struct InternedToken {
    kind: TokenKind,
    span: Span,
    spur: Spur, // The interned string
    data: TokenData, // Additional data for some tokens
}

impl PartialEq for InternedToken {
    fn eq(&self, other: &Self) -> bool {
        // A hex literal and decimal literal equal if they have the same value
        match (self.kind(), other.kind()) {
            (TokenKind::HexInteger, TokenKind::Integer) => {
                self.data.unwrap_byte() == other.data.unwrap_byte()
            }
            (TokenKind::Integer, TokenKind::HexInteger) => {
                self.data.unwrap_byte() == other.data.unwrap_byte()
            }
            _ => {
                debug_assert!(self.kind == other.kind);
                self.spur == other.spur
            }
        }
    }
}

impl Eq for InternedToken {}

impl std::hash::Hash for InternedToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.spur.hash(state);
    }
}

impl InternedToken {
    pub fn new(kind: TokenKind, span: Span, spur: Spur, data: TokenData) -> Self {
        Self {
            kind,
            span,
            spur,
            data,
        }
    }

    #[inline]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub fn spur(&self) -> Spur {
        self.spur
    }

    #[inline]
    pub fn data(&self) -> &TokenData {
        &self.data
    }

    pub fn text<'a>(&'a self, rodeo: &'a Rodeo) -> &'a str {
        rodeo.resolve(&self.spur)
    }
}

/// A token emitted by the lexer.
#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TokenKind {
    #[error]
    Error,

    #[regex(r"[ \t\n\f]+")]
    Whitespace,

    // Comments start with a # and go to the end of the line
    #[regex(r"#[^\r\n]*")]
    Comment,

    // Keywords
    #[token("IMPORT")]
    ImportKW,

    // ---- Atoms ----
    // Decimal integer
    #[regex(r"[+-]?[0-9]+", priority = 2)]
    Integer,

    // Hexadecimal integer
    #[regex(r"[+-]?0[xX][0-9a-fA-F]+")]
    HexInteger,

    // String with " "
    #[regex(r#""[^"]*""#)]
    String,

    // String with ' '
    #[regex(r#"'[^']*'"#)]
    RawString,

    // Brainfuck block. backticks with any characters inside. No escaping.
    #[regex(r#"`[^`]*`"#)]
    Brainfuck,

    // Macro input. { } with any characters inside (including newlines). No escaping.
    #[regex(r#"\{[^}]*\}"#)]
    MacroInput,

    // ---- Identifiers ----
    // Almost anything can be an identifier. Some identifier are reserved
    // - Identifier can not start with "-0[xX]" because that would more closely match a hex number
    #[regex(r"[^ ;\t\n\f#@\?\(\)\[\]\{{\}}\d][^ \t\n\f#@\?\(\)\[\]\{{\}};]*")]
    Identifier,

    // Single lowercase letter
    #[regex(r"[a-z]", priority = 2)]
    NamedByte,

    // Single uppercase letter
    #[regex(r"[A-Z]", priority = 2)]
    NamedQuotation,

    // Ignored input byte
    #[token("@")]
    UnnamedByte,

    // Ignored input quotation
    #[token("?")]
    UnnamedQuotation,

    // ---- Symbols ----
    #[token("==")]
    Substitution,

    #[token("==?")]
    Generation,

    #[token("==!")]
    Execution,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token(";")]
    Semicolon,

    #[token(".")]
    Dot,
}

impl TokenKind {
    /// Returns a static slice of which tokens are atoms.
    ///
    /// Atoms are tokens that can be used within the body of a definition or a quotation.
    pub const fn atomics() -> &'static [TokenKind] {
        &[
            TokenKind::Integer,
            TokenKind::HexInteger,
            TokenKind::String,
            TokenKind::RawString,
            TokenKind::MacroInput,
            TokenKind::NamedByte,
            TokenKind::NamedQuotation,
            TokenKind::Identifier,
            TokenKind::Brainfuck,
        ]
    }

    /// Returns true if the token is an atom.
    ///
    /// Atoms are tokens that can be used within the body of a definition or a quotation.
    pub fn is_atomic(&self) -> bool {
        Self::atomics().contains(self)
    }

    /// Returns a static slice of which tokens are trivia.
    ///
    /// Trivia are tokens that are to be (mostly) ignored by the parser.
    pub const fn trivia() -> &'static [TokenKind] {
        &[TokenKind::Whitespace, TokenKind::Comment]
    }

    /// Returns true is a token is trivia.
    pub fn is_trivia(&self) -> bool {
        Self::trivia().contains(self)
    }
}

/// Some tokens have additional information.
/// Currently this is either an interned string or a number.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenData {
    None,
    Byte(u8),
    String(Spur),
}

impl TokenData {
    pub fn is_none(&self) -> bool {
        matches!(self, TokenData::None)
    }

    pub fn is_byte(&self) -> bool {
        matches!(self, TokenData::Byte(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, TokenData::String(_))
    }

    pub fn unwrap_byte(&self) -> u8 {
        match self {
            TokenData::Byte(i) => *i,
            _ => panic!("Called TokenData::unwrap_integer on a non-integer"),
        }
    }

    pub fn unwrap_string(&self) -> Spur {
        match self {
            TokenData::String(s) => *s,
            _ => panic!("Called TokenData::unwrap_string on a non-string"),
        }
    }
}

#[cfg(test)]
mod test {
    use logos::Logos;
    use proptest::prelude::*;

    use crate::lexer::token::TokenKind;

    proptest! {
        #[test]
        fn never_crash(s in "\\PC*") {
            TokenKind::lexer(&s);
        }

        // Verifies that [a-z] generates a NamedByte token and not an Identifier
        #[test]
        fn named_byte(s in "[a-z]") {
            let mut lexer = TokenKind::lexer(&s);
            assert_eq!(lexer.next(), Some(TokenKind::NamedByte));
            assert_eq!(lexer.next(), None);
        }

        // Verifies that [A-Z] generates a NamedQuotation token and not an Identifier
        #[test]
        fn named_quotation(s in "[A-Z]") {
            let mut lexer = TokenKind::lexer(&s);
            assert_eq!(lexer.next(), Some(TokenKind::NamedQuotation));
            assert_eq!(lexer.next(), None);
        }
    }
}
