use codespan_reporting::diagnostic::Diagnostic;
use colored::Colorize;
use snailquote::UnescapeError;

use crate::Span;

const ICE_NOTE: &str =
    "This is a compiler error and should not have happened. Please report this bug.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizerError {
    ICEEmptyStringAsInteger(Span),
    NegativeInteger(Span, u8),
    LargeInteger(Span, u8),
    ICEValidIntegerFailed(Span),
    ICEEmptyStringAsHex(Span),
    NegativeHex(Span, u8),
    LargeHex(Span, u8),
    ICEValidHexFailed(Span),
    ICEStringCouldNotBeTrimmed(Span),
    InvalidEscapeSequence(Span, UnescapeError),
    NewlineInString(Span),
    NonAsciiString(Span, Span),
}

impl TokenizerError {
    pub fn code(&self) -> &'static str {
        use TokenizerError::*;

        match self {
            ICEEmptyStringAsInteger(_) => "I000",
            NegativeInteger(_, _) => "E001",
            LargeInteger(_, _) => "E002",
            ICEValidIntegerFailed(_) => "I003",
            ICEEmptyStringAsHex(_) => "I004",
            NegativeHex(_, _) => "E005",
            LargeHex(_, _) => "E006",
            ICEValidHexFailed(_) => "I007",
            ICEStringCouldNotBeTrimmed(_) => "I008",
            InvalidEscapeSequence(_, _) => "E009",
            NewlineInString(_) => "E010",
            NonAsciiString(_, _) => "E011",
        }
    }

    pub fn message(&self) -> &'static str {
        use TokenizerError::*;

        match self {
            ICEEmptyStringAsInteger(_) => {
                "Internal Compiler Error: Attempted to parse an empty string as integer"
            }
            NegativeInteger(_, _) => "Invalid byte: Negative numbers are not supported",
            LargeInteger(_, _) => "Invalid byte: Number is too large to store in a byte",
            ICEValidIntegerFailed(_) => {
                "Internal Compiler Error: Failed to parse an integer string that should have succeeded"
            }
            ICEEmptyStringAsHex(_) => {
                "Internal Compiler Error: Attempted to parse an empty string as hex integer"
            }
            NegativeHex(_, _) => "Invalid byte: Negative numbers are not supported",
            LargeHex(_, _) => "Invalid byte: Number is too large to store in a byte",
            ICEValidHexFailed(_) => {
                "Internal Compiler Error: Failed to parse a hex string that should have succeeded"
            }
            ICEStringCouldNotBeTrimmed(_) => {
                "Internal Compiler Error: Failed to trim a stringy type"
            }
            InvalidEscapeSequence(_, _) => "Invalid escape sequence in string.",
            NewlineInString(_) => "Newlines are not allowed in strings.",
            NonAsciiString(_, _) => "Non-ASCII characters are not allowed in strings.",
        }
    }
}

impl From<TokenizerError> for Diagnostic<usize> {
    fn from(err: TokenizerError) -> Self {
        use TokenizerError::*;

        // message & code are handled by the respective methods
        match err.clone() {
            ICEEmptyStringAsInteger(span) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![
                    span.primary_label("Attempted to parse an empty string as integer")
                ]),
            NegativeInteger(span, inverse) => Diagnostic::error().with_labels(vec![span
                .primary_label(format!(
                    "Consider using the arithmetic inverse instead: {}",
                    inverse
                ))]),
            LargeInteger(span, modulo) => Diagnostic::error().with_labels(vec![span
                .primary_label(format!(
                    "Consider using the modulo operator instead: {}",
                    modulo
                ))]),
            ICEValidIntegerFailed(span) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![span.primary_label(
                    "Failed to parse an integer string that should have succeeded",
                )]),
            ICEEmptyStringAsHex(span) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![
                    span.primary_label("Attempted to parse an empty string as hex integer")
                ]),
            NegativeHex(span, inverse) => Diagnostic::error().with_labels(vec![span
                .primary_label(format!(
                    "Consider using the arithmetic inverse instead: {}",
                    inverse.to_string().yellow()
                ))]),
            LargeHex(span, modulo) => {
                Diagnostic::error().with_labels(vec![span.primary_label(format!(
                    "Consider using the modulo operator instead: {}",
                    modulo.to_string().yellow()
                ))])
            }
            ICEValidHexFailed(span) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![span.primary_label(
                    "Failed to parse a hex string that should have succeeded",
                )]),
            ICEStringCouldNotBeTrimmed(span) => Diagnostic::error()
                .with_notes(vec![ICE_NOTE.to_string()])
                .with_labels(vec![span.primary_label("Failed to trim a String type")]),
            InvalidEscapeSequence(span, e) => {
                Diagnostic::error().with_labels(vec![span.primary_label(e.to_string())])
            }
            NewlineInString(span) => Diagnostic::error().with_labels(vec![
                span.primary_label(format!("Consider using {} instead", "\\n".yellow()))
            ]),
            NonAsciiString(span, char) => Diagnostic::error().with_labels(vec![
                span.primary_label("Strings with non-ascii characters are not yet supported")
            ]),
        }
        .with_message(err.message())
        .with_code(err.code())
    }
}
