//! A lexer for serotonin implemented using [`logos`](https://crates.io/crates/logos).
//!
//! Since the language is so simple a lexer can almost completely parse the language.
//! The only thing that can not be handled by the lexer is nested quotations.
use std::{ops::Range, rc::Rc};

use lasso::Rodeo;
use logos::Logos;
use num::{BigInt, ToPrimitive};

use crate::{InternedToken, Span};

use super::{
    token::{Token, TokenData},
    TokenizerError,
};

pub fn lex(
    input: &str,
    file_id: usize,
    rodeo: &mut Rodeo,
) -> (Vec<Rc<InternedToken>>, Vec<TokenizerError>) {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    for (token, range) in Token::lexer(input).spanned() {
        let slice = &input[range.clone()];

        match create_interned_token(token, range, slice, file_id, rodeo) {
            Ok(token) => tokens.push(Rc::new(token)),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    (tokens, diagnostics)
}

// create token data
fn create_interned_token(
    token: Token,
    range: Range<usize>,
    slice: &str,
    file_id: usize,
    rodeo: &mut Rodeo,
) -> Result<InternedToken, TokenizerError> {
    let span = Span::from_range(range, file_id);

    let data: TokenData = match token {
        Token::Integer => TokenData::Integer(lex_integer(slice, span)?),
        Token::HexInteger => TokenData::Integer(lex_hex(slice, span)?),
        Token::String | Token::RawString => {
            no_newlines(slice, span)?;
            let slice = &unescape(slice, span)?;
            ascii_only(slice, span)?;

            let spur = rodeo.get_or_intern(slice);
            TokenData::String(spur)
        }
        Token::Brainfuck => {
            let slice = trim(slice, span)?;
            no_newlines(slice, span)?;

            let spur = rodeo.get_or_intern(slice);
            TokenData::String(spur)
        }
        Token::MacroInput => {
            let slice = trim(slice, span)?;

            let spur = rodeo.get_or_intern(slice);
            TokenData::String(spur)
        }
        Token::NamedByte | Token::NamedQuotation | Token::Identifier => {
            let spur = rodeo.get_or_intern(slice);
            TokenData::String(spur)
        }
        _ => TokenData::None,
    };

    Ok(InternedToken::new(
        token,
        span,
        rodeo.get_or_intern(slice),
        data,
    ))
}

/// Parses a hex integer that matches "0[xX][0-9a-fA-F]+"
fn lex_hex(slice: &str, span: Span) -> Result<u8, TokenizerError> {
    if slice.is_empty() {
        return Err(TokenizerError::ICEEmptyStringAsHex(span));
    }

    // We cannot represent negative numbers. Suggest the additive inverse instead
    if let Some(slice) = slice.strip_prefix('-') {
        match lex_hex(
            slice,
            Span::new(span.start() + 1, span.end(), span.file_id()),
        ) {
            Ok(magnitude) => {
                // Suggest using 256 - n instead
                let n = if magnitude == 0 {
                    0
                } else {
                    256 - magnitude as i16
                } as u8;

                Err(TokenizerError::NegativeHex(span, n))
            }
            Err(TokenizerError::LargeInteger(span, n)) => {
                // A large negative number.
                let span = Span::new(span.start() + 1, span.end(), span.file_id());
                Err(TokenizerError::LargeHex(span, n))
            }
            Err(e) => Err(e),
        }
    } else {
        // We need to optionally trim `+?0[xX]` from the start of the string
        let slice = if slice.starts_with("0x") || slice.starts_with("0X") {
            &slice[2..]
        } else if slice.starts_with('+') {
            &slice[3..]
        } else {
            slice
        };

        if slice.len() > 2 {
            // Too large: we can only store a single byte
            let b = BigInt::parse_bytes(slice.as_bytes(), 16).unwrap();
            let remainder: BigInt = b % 256;
            let n = remainder.to_u8().unwrap();

            return Err(TokenizerError::LargeHex(span, n));
        }

        // Parse the number. Now we should be confident that the number is valid u8
        match u8::from_str_radix(slice, 16) {
            Ok(n) => Ok(n),
            Err(_) => Err(TokenizerError::ICEValidHexFailed(span)),
        }
    }
}

/// Parses an integer that matches "[+-]?[0-9]+"
fn lex_integer(slice: &str, span: Span) -> Result<u8, TokenizerError> {
    if slice.is_empty() {
        return Err(TokenizerError::ICEEmptyStringAsInteger(span));
    }

    // We cannot represent negative numbers. Suggest the additive inverse instead
    if let Some(slice) = slice.strip_prefix('-') {
        match lex_integer(
            slice,
            Span::new(span.start() + 1, span.end(), span.file_id()),
        ) {
            Ok(magnitude) => {
                // Suggest using 256 - n instead
                let n = if magnitude == 0 {
                    0
                } else {
                    256 - magnitude as i16
                } as u8;

                Err(TokenizerError::NegativeInteger(span, n))
            }
            Err(TokenizerError::LargeInteger(span, n)) => {
                // A large negative number.
                let span = Span::new(span.start() + 1, span.end(), span.file_id());
                Err(TokenizerError::LargeInteger(span, n))
            }
            Err(e) => Err(e),
        }
    } else {
        // We need to optionally trim `+?` from the start of the string
        let slice = slice.strip_prefix('+').unwrap_or(slice);

        if slice.len() > 3 {
            // Too large: we can only store a single byte
            let b = BigInt::parse_bytes(slice.as_bytes(), 10).unwrap();
            let remainder: BigInt = b % 256;
            let n = remainder.to_u8().unwrap();

            return Err(TokenizerError::LargeInteger(span, n));
        }

        // Parse the number. Now we should be confident that the number is valid u8
        match slice.parse::<u8>() {
            Ok(n) => Ok(n),
            Err(_) => Err(TokenizerError::ICEValidIntegerFailed(span)),
        }
    }
}

/// Trim starting and ending characters from String, RawString, or MacroInput
fn trim(slice: &str, span: Span) -> Result<&str, TokenizerError> {
    if slice.len() < 2 {
        // Compiler error
        return Err(TokenizerError::ICEStringCouldNotBeTrimmed(span));
    }

    Ok(&slice[1..slice.len() - 1])
}

/// Validate a string does not contain any newlines
fn no_newlines(slice: &str, span: Span) -> Result<(), TokenizerError> {
    match slice
        .char_indices()
        .find_map(|(i, c)| if c == '\n' { Some(i) } else { None })
    {
        Some(i) => {
            let char: Span = Span::new(span.start() + i, span.start() + i + 1, span.file_id());
            Err(TokenizerError::NewlineInString(span, char))
        }
        None => Ok(()),
    }
}

/// Unescape a string using the snailquote crate
fn unescape(slice: &str, span: Span) -> Result<String, TokenizerError> {
    match snailquote::unescape(slice) {
        Ok(s) => Ok(s),
        Err(e) => Err(TokenizerError::InvalidEscapeSequence(span, e)),
    }
}

/// Validate a string only contains ascii characters
fn ascii_only(slice: &str, span: Span) -> Result<(), TokenizerError> {
    for (i, c) in slice.chars().enumerate() {
        if !c.is_ascii() {
            let char: Span = Span::new(span.start() + i, span.start() + i, span.file_id());
            return Err(TokenizerError::NonAsciiString(span, char));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use logos::Logos;
    use proptest::prelude::*;

    use crate::{
        lexer::{
            lex::{ascii_only, lex_hex, lex_integer, no_newlines},
            token::Token,
            TokenizerError,
        },
        Span,
    };

    proptest! {
        // Verifies integers can be parsed any size, and optionally signed
        #[test]
        fn test_integer(s in "[+-]?[0-9]+") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::Integer));
            assert_eq!(lexer.next(), None);
        }

        // Negative integers should tokenize but will emit an error
        #[test]
        fn test_negative_integer(s in "-[0-9]{1,2}") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::Integer));
            let span = Span::from_range(lexer.span(), 0);
            let slice = lexer.slice();
            assert_eq!(lexer.next(), None);

            let err = lex_integer(slice, span).unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, TokenizerError::NegativeInteger(..)));
        }

        // Large negative integers should tokenize but will emit an error
        #[test]
        fn test_negative_large_integer(s in "-[0-9]{4,}") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::Integer));
            let span = Span::from_range(lexer.span(), 0);
            let slice = lexer.slice();
            assert_eq!(lexer.next(), None);

            let err = lex_integer(slice, span).unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, TokenizerError::LargeInteger(..)));
        }

        // Verifies hex can be parsed any size, and optionally signed
        #[test]
        fn test_hex(s in "[+-]?0[xX][0-9a-fA-F]+") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::HexInteger));
            assert_eq!(lexer.next(), None);
        }

        // Negative hex should tokenize but will emit an error
        #[test]
        fn test_negative_hex(s in "-0[xX][0-9a-fA-F]{1,2}") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::HexInteger));
            let span = Span::from_range(lexer.span(), 0);
            let slice = lexer.slice();
            assert_eq!(lexer.next(), None);

            let err = lex_hex(slice, span).unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, TokenizerError::NegativeHex(..)));
        }

        // Large negative hex should tokenize but will emit an error
        #[test]
        fn test_negative_large_hex(s in "-0[xX][0-9a-fA-F]{4,}") {
            let mut lexer = Token::lexer(&s);
            assert_eq!(lexer.next(), Some(Token::HexInteger));
            let span = Span::from_range(lexer.span(), 0);
            let slice = lexer.slice();
            assert_eq!(lexer.next(), None);

            let err = lex_hex(slice, span).unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, TokenizerError::LargeHex(..)));
        }

        // Verify the ascii_only function works
        #[test]
        fn test_ascii_only(s in "[[:ascii:]]+") {
            let span = Span::new(0, s.len(), 0);
            ascii_only(&s, span).unwrap();
        }

        // Verify the ascii_only function fails on non-ascii characters
        #[test]
        fn test_non_ascii_only(s in "[^[:ascii:]]+") {
            let span = Span::new(0, s.len(), 0);
            let err = ascii_only(&s, span).unwrap_err();
            println!("{:?}", err);
            assert!(matches!(err, TokenizerError::NonAsciiString(..)));
        }

        // Verify no newlines works
        #[test]
        fn test_no_newlines(s in "[^\n]+") {
            let span = Span::new(0, s.len(), 0);
            no_newlines(&s, span).unwrap();
        }

        // Verify no newlines when a string contains a newline
        #[test]
        fn test_newline_in_string(s in "[^\n]+\\n[^\n]*") {
            let span = Span::new(0, s.len(), 0);
            no_newlines(&s, span).unwrap_err();
        }
    }

    // While parsing a string with a newline make sure the error returns the correct span
    #[test]
    fn test_newline_in_string_span() {
        let s = r#""foo
        bar""#;
        let mut lexer = Token::lexer(s);
        assert_eq!(lexer.next(), Some(Token::String));
        let span = Span::from_range(lexer.span(), 0);
        let slice = lexer.slice();
        assert_eq!(lexer.next(), None);

        let err = no_newlines(slice, span).unwrap_err();
        let TokenizerError::NewlineInString( string_span, newline_span ) = err else {
            panic!("Expected a newline error");
        };

        assert_eq!(string_span, span);
        assert_eq!(newline_span, Span::new(4, 5, 0));
    }

    // While parsing a string with a unicode character make sure the error returns the correct span
    #[test]
    fn test_unicode_in_string_span() {
        let s = r#""foo🚀bar""#;
        let mut lexer = Token::lexer(s);
        assert_eq!(lexer.next(), Some(Token::String));
        let span = Span::from_range(lexer.span(), 0);
        let slice = lexer.slice();
        assert_eq!(lexer.next(), None);

        let err = ascii_only(slice, span).unwrap_err();
        assert!(matches!(err, TokenizerError::NonAsciiString(..)));
    }
}
