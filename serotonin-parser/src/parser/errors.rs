use codespan_reporting::diagnostic::Diagnostic;

use crate::{Span, TokenKind, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken {
        found: Token,
        expected: Expectations,
    },
    UnexpectedEOF {
        eof: Span,
        expected: Expectations,
    },
}

impl From<ParseError> for Diagnostic<usize> {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::UnexpectedToken { found, expected } => {
                let message = "Error Unexpected InternedToken".to_string();
                Diagnostic::error()
                    .with_message(message)
                    .with_labels(vec![found.span().primary_label(format!(
                        "Expected {} found {:?}",
                        expected.into_message(),
                        found.kind()
                    ))])
            }
            ParseError::UnexpectedEOF { eof, expected } => {
                let message = "Error Unexpected EOF".to_string();
                Diagnostic::error().with_message(message).with_labels(vec![
                    eof.primary_label(format!("Expected {} found EOF", expected.into_message()))
                ])
            }
        }
    }
}

// Expectations
#[derive(Debug, Clone)]
pub enum Expectations {
    Any,
    Exactly(TokenKind),
    OneOf(Vec<TokenKind>),
}

impl PartialEq for Expectations {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expectations::Any, Expectations::Any) => true,
            (Expectations::Exactly(t), Expectations::Exactly(o)) => t == o,
            (Expectations::OneOf(v), Expectations::OneOf(o)) => {
                // The order of the tokens doesn't matter
                v.iter().all(|t| o.contains(t)) && o.iter().all(|t| v.contains(t))
            }
            _ => false,
        }
    }
}

impl Eq for Expectations {}

impl Expectations {
    fn into_message(self) -> String {
        match self {
            Expectations::Any => "anything".to_string(),
            Expectations::Exactly(token) => {
                // TODO: Create token's display impl
                format!("{:?}", token)
            }
            Expectations::OneOf(tokens) => {
                format!("one of [{:?}]", tokens)
            }
        }
    }
}
