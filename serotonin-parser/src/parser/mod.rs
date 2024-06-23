mod definition;
mod errors;
mod import;
mod module;
mod stack;

use codespan_reporting::diagnostic::Diagnostic;
use lasso::Spur;

use crate::{
    ast::{Definition, Module},
    Span, Token, TokenKind,
};

use errors::ParseError;

use self::errors::Expectations;

/// Parses a module from a list of tokens
///
/// Requires the module name and span to be passed as additional arguments
pub fn parse_module(
    tokens: &[Token],
    file_id: usize,
    name: Spur,
) -> Result<(Module, Vec<Diagnostic<usize>>), ParseError> {
    let mut parser = Parser::new(tokens, file_id);
    Ok((parser.parse_module(name)?, parser.emits))
}

// Parses a single definition. This is helpful for testing
pub fn parse_definition(tokens: &[Token]) -> Result<Definition, ParseError> {
    let mut parser = Parser::new(tokens, 0);
    parser.parse_definition()
}

pub struct Parser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) index: usize,        // Index into the `tokens` array
    pub(crate) source_index: usize, // span().end() of the previous token
    pub(crate) file_id: usize, // File ID of the current file. The parser does not cross file boundaries
    pub(crate) emits: Vec<Diagnostic<usize>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], file_id: usize) -> Self {
        Self {
            tokens,
            index: 0,
            source_index: 0,
            file_id,
            emits: Vec::new(),
        }
    }

    /// Returns the next token without consuming it
    pub(crate) fn peek(&mut self) -> Option<Token> {
        self.tokens.get(self.index).cloned()
    }

    /// Returns true if the next token is the given kind
    pub(crate) fn peek_is(&mut self, token: TokenKind) -> bool {
        self.peek().map(|t| t.kind() == token).unwrap_or(false)
    }

    /// Returns the next token and consumes it
    pub(crate) fn next(&mut self) -> Option<Token> {
        let next = self.peek()?;
        self.index += 1;
        self.source_index = next.span().end();
        Some(next)
    }

    /// Consumes the next token if it matches the expected token
    ///
    /// Errors if the next token was not the expected token
    pub(crate) fn expect(&mut self, token: TokenKind) -> Result<Token, ParseError> {
        let next = self.next().ok_or(ParseError::UnexpectedEOF {
            eof: Span::new(self.source_index, self.source_index, self.file_id),
            expected: Expectations::Exactly(token),
        })?;

        if next.kind() == token {
            Ok(next)
        } else {
            Err(ParseError::UnexpectedToken {
                found: next,
                expected: Expectations::Exactly(token),
            })
        }
    }

    /// Consumes the next token if one of the given tokens matches
    pub(crate) fn expect_one_of(&mut self, tokens: &[TokenKind]) -> Result<Token, ParseError> {
        let next = self.next().ok_or(ParseError::UnexpectedEOF {
            eof: Span::new(self.source_index, self.source_index, self.file_id),
            expected: Expectations::OneOf(tokens.to_vec()),
        })?;

        if tokens.contains(&next.kind()) {
            Ok(next)
        } else {
            Err(ParseError::UnexpectedToken {
                found: next,
                expected: Expectations::OneOf(tokens.to_vec()),
            })
        }
    }

    /// Collects 0 or more tokens of the given kind, separated by trivia
    pub(crate) fn sep(&mut self, kind: &[TokenKind]) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.peek() {
            if kind.contains(&token.kind()) {
                tokens.push(self.next().unwrap());
                self.skip_trivia();
            } else {
                break;
            }
        }
        tokens
    }

    /// Skip a token if it matches one of the given tokens
    pub(crate) fn skip(&mut self, token: &[TokenKind]) {
        while let Some(next) = self.peek() {
            if !token.contains(&next.kind()) {
                break;
            }

            self.next().unwrap();
        }
    }

    /// Skip all trivia tokens
    pub(crate) fn skip_trivia(&mut self) {
        self.skip(TokenKind::trivia());
    }
}
