use codespan_reporting::diagnostic::Diagnostic;

use crate::{Span, Token, ICE_NOTE};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticError {
    ICENamedByteHasLengthNotOne(Token),
    ICENamedQuotationHasLengthNotOne(Token),
    ICEByteMissingValue(Token),
}

impl SemanticError {
    pub fn message(&self) -> &'static str {
        match self {
            SemanticError::ICENamedByteHasLengthNotOne(_) => {
                "Internal Compiler Error: NamedByte has length not equal to one"
            }
            SemanticError::ICENamedQuotationHasLengthNotOne(_) => {
                "Internal Compiler Error: NamedQuotation has length not equal to one"
            }
            SemanticError::ICEByteMissingValue(_) => {
                "Internal Compiler Error: Byte is missing it's value"
            }
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            SemanticError::ICENamedByteHasLengthNotOne(_) => "I200",
            SemanticError::ICENamedQuotationHasLengthNotOne(_) => "I201",
            SemanticError::ICEByteMissingValue(_) => "I202",
        }
    }
}

impl From<SemanticError> for Diagnostic<usize> {
    fn from(value: SemanticError) -> Self {
        use SemanticError as SE;

        let code = value.code();
        let message = value.message();

        match value {
            SE::ICENamedByteHasLengthNotOne(t) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![t.span().primary_label(
                    "NamedByte length is not equal to one, lexing should have caught this",
                )]),
            SE::ICENamedQuotationHasLengthNotOne(t) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![t.span().primary_label(
                    "NamedQuotation length is not equal to one, lexing should have caught this",
                )]),
            SE::ICEByteMissingValue(t) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![t.span().primary_label("Byte is missing it's value")]),
        }
        .with_code(code)
        .with_message(message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticWarning {
    SpecificQuotationsNotSupported(Span),
}

impl SemanticWarning {
    pub fn message(&self) -> &'static str {
        match self {
            SemanticWarning::SpecificQuotationsNotSupported(_) => {
                "Specific quotation constraints are not yet supported"
            }
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            SemanticWarning::SpecificQuotationsNotSupported(_) => "W203",
        }
    }
}

impl From<SemanticWarning> for Diagnostic<usize> {
    fn from(value: SemanticWarning) -> Self {
        use SemanticWarning as SW;

        let code = value.code();
        let message = value.message();

        match value {
            SW::SpecificQuotationsNotSupported(span) => Diagnostic::warning()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![span.primary_label(
                    "Specific quotation constraints are not yet supported, they will be ignored",
                )]),
        }
        .with_code(code)
        .with_message(message)
    }
}
