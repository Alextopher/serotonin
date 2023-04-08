use std::rc::Rc;

use codespan_reporting::diagnostic::Diagnostic;

use crate::{InternedToken, Span, Token};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    UnexpectedToken {
        found: Rc<InternedToken>,
        expected: Expectations,
    },
    UnexpectedEOF {
        eof: Span,
        expected: Expectations,
    },
}

// Converts a parser error into a diagnostic
impl ParseError {
    pub fn into_diagnostic(self) -> Diagnostic<usize> {
        match self {
            ParseError::UnexpectedToken { found, expected } => {
                let message = "Error Unexpected Token".to_string();
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expectations {
    Any,
    Exactly(Token),
    OneOf(Vec<Token>),
}

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
