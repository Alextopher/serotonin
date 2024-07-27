use std::sync::Arc;

use codespan_reporting::diagnostic::Diagnostic;
use colored::Colorize;
use snailquote::UnescapeError;

use crate::{Span, ICE_NOTE};

#[derive(Debug, Clone, PartialEq)]
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
    InvalidEscapeSequence(Span, Arc<UnescapeError>),
    NewlineInString(Span, Span),
    NonAsciiString(Span, Span),
    UnknownToken(Span), // generic parsing error
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
            NewlineInString(_, _) => "E010",
            NonAsciiString(_, _) => "E011",
            UnknownToken(_) => "E012",
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
            NewlineInString(_, _) => "Newlines are not allowed in strings.",
            NonAsciiString(_, _) => "Non-ASCII characters are not allowed in strings.",
            UnknownToken(_) => "Invalid token.",
        }
    }
}

impl From<(Span, UnescapeError)> for TokenizerError {
    fn from((span, err): (Span, UnescapeError)) -> Self {
        TokenizerError::InvalidEscapeSequence(span, Arc::new(err))
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
                    inverse.to_string().yellow()
                ))]),
            LargeInteger(span, modulo) => Diagnostic::error().with_labels(vec![span
                .primary_label(format!(
                    "Consider using the result after overflow: {}",
                    modulo.to_string().yellow()
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
            NewlineInString(span, newline) => Diagnostic::error().with_labels(vec![
                span.primary_label(format!(
                    "Consider using an escape code instead: {}",
                    "\\n".yellow()
                )),
                newline.secondary_label("Newline found here"),
            ]),
            NonAsciiString(span, char) => Diagnostic::error().with_labels(vec![
                span.primary_label("Strings with non-ascii characters are not yet supported"),
                char.secondary_label("Non-ascii character found here"),
            ]),
            UnknownToken(span) => {
                Diagnostic::error().with_labels(vec![span.primary_label("Invalid token.")])
            }
        }
        .with_message(err.message())
        .with_code(err.code())
    }
}

// Test the output of every error
#[cfg(test)]
mod test {
    use codespan_reporting::{diagnostic::Diagnostic, files::SimpleFiles, term};

    use crate::Span;

    use super::TokenizerError;

    fn print_error(files: SimpleFiles<&str, &str>, err: TokenizerError) {
        let mut writer = std::io::sink();
        let config = codespan_reporting::term::Config::default();

        let diagnostic: Diagnostic<usize> = err.into();
        term::emit(&mut writer, &config, &files, &diagnostic).unwrap();
    }

    #[test]
    fn test_ice_empty_string_as_integer() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "This is a test file");

        let err = TokenizerError::ICEEmptyStringAsInteger(Span::new(0, 0, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_negative_integer() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "main == -10;");

        let err = TokenizerError::NegativeInteger(Span::new(8, 11, file_id), 246);
        print_error(files, err);
    }

    #[test]
    fn test_large_integer() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "main == 300;");

        let err = TokenizerError::LargeInteger(Span::new(8, 12, file_id), 44);
        print_error(files, err);
    }

    #[test]
    fn test_ice_valid_integer_failed() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "123");

        let err = TokenizerError::ICEValidIntegerFailed(Span::new(0, 3, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_ice_empty_string_as_hex() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "This is a test file");

        let err = TokenizerError::ICEEmptyStringAsHex(Span::new(0, 0, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_negative_hex() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "main == -0x10;");

        let err = TokenizerError::NegativeHex(Span::new(8, 14, file_id), 0xF0);
        print_error(files, err);
    }

    #[test]
    fn test_large_hex() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "main == 0x100;");

        let err = TokenizerError::LargeHex(Span::new(8, 15, file_id), 0);
        print_error(files, err);
    }

    #[test]
    fn test_ice_valid_hex_failed() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "0x123");

        let err = TokenizerError::ICEValidHexFailed(Span::new(0, 5, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_ice_string_could_not_be_trimmed() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "'a'");

        let err = TokenizerError::ICEStringCouldNotBeTrimmed(Span::new(0, 3, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let mut files = SimpleFiles::new();
        let text = r#""\m""#;

        let file_id = files.add("test", text);

        let err = TokenizerError::InvalidEscapeSequence(
            Span::new(9, 11, file_id),
            snailquote::unescape(text).unwrap_err().into(),
        );

        print_error(files, err);
    }

    #[test]
    fn test_newline_in_string() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "'Hello\nworld'");

        let err =
            TokenizerError::NewlineInString(Span::new(0, 13, file_id), Span::new(6, 7, file_id));
        print_error(files, err);
    }

    #[test]
    fn test_non_ascii_string() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", "'Héllo world'");

        let err =
            TokenizerError::NonAsciiString(Span::new(0, 14, file_id), Span::new(1, 2, file_id));
        print_error(files, err);
    }
}
