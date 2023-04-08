mod definition;
mod errors;
mod import;
mod module;
mod stack;

use std::rc::Rc;

use codespan_reporting::diagnostic::Diagnostic;
use lasso::Spur;

use crate::{
    ast::{Definition, Module},
    InternedToken, Span, Token,
};

use errors::ParseError;

use self::errors::Expectations;

/// Parses a module from a list of tokens
///
/// Requires the module name and span to be passed as additional arguments
pub fn parse_module(
    tokens: &[Rc<InternedToken>],
    span: Span,
    name: Spur,
) -> Result<(Module, Vec<Diagnostic<usize>>), ParseError> {
    debug_assert_eq!(
        span.start(),
        0,
        "Modules must start at the beginning of the file"
    );

    let mut parser = Parser::new(tokens, span.file_id());
    Ok((parser.parse_module(span, name)?, parser.emits))
}

// Parses a single definition. This is helpful for testing
pub fn parse_definition(tokens: &[Rc<InternedToken>]) -> Result<Definition, ParseError> {
    let mut parser = Parser::new(tokens, 0);
    parser.parse_definition()
}

pub struct Parser<'a> {
    pub(crate) tokens: &'a [Rc<InternedToken>],
    pub(crate) index: usize,
    pub(crate) source_index: usize, // The start of the current token in the source code
    pub(crate) file_id: usize,
    pub(crate) emits: Vec<Diagnostic<usize>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Rc<InternedToken>], file_id: usize) -> Self {
        Self {
            tokens,
            index: 0,
            source_index: 0,
            file_id,
            emits: Vec::new(),
        }
    }

    /// Returns the next token without consuming it
    pub(crate) fn peek(&mut self) -> Option<Rc<InternedToken>> {
        self.tokens.get(self.index).cloned()
    }

    /// Returns true if the next token is the given kind
    pub(crate) fn peek_is(&mut self, token: Token) -> bool {
        self.peek().map(|t| t.kind() == token).unwrap_or(false)
    }

    /// Returns the next token and consumes it
    pub(crate) fn next(&mut self) -> Option<Rc<InternedToken>> {
        let next = self.peek()?;
        self.index += 1;
        self.source_index = next.span().end();
        Some(next)
    }

    /// Consumes the next token if it matches the expected token
    ///
    /// Errors if the next token was not the expected token
    pub(crate) fn expect(&mut self, token: Token) -> Result<Rc<InternedToken>, ParseError> {
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
    pub(crate) fn expect_one_of(
        &mut self,
        tokens: &[Token],
    ) -> Result<Rc<InternedToken>, ParseError> {
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

    /// Collects 0 or more tokens of the given kind
    // pub(crate) fn collect(&mut self, kind: Token) -> Vec<Rc<InternedToken>> {
    //     let mut tokens = Vec::new();
    //     while let Some(token) = self.peek() {
    //         if token.kind() == kind {
    //             tokens.push(self.next().unwrap());
    //         } else {
    //             break;
    //         }
    //     }
    //     tokens
    // }

    /// Collects 0 or more tokens of the given kind, separated by trivia
    pub(crate) fn sep(&mut self, kind: &[Token]) -> Vec<Rc<InternedToken>> {
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
    pub(crate) fn skip(&mut self, token: &[Token]) {
        while let Some(next) = self.peek() {
            if !token.contains(&next.kind()) {
                break;
            }

            self.next().unwrap();
        }
    }

    /// Skip all trivia tokens
    pub(crate) fn skip_trivia(&mut self) {
        self.skip(Token::trivia());
    }
}
