use crate::{ast::Imports, TokenKind};

use super::{
    errors::{Expectations, ParseError},
    Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn optional_imports(&mut self) -> Option<Result<Imports, ParseError>> {
        if self.peek_is(TokenKind::ImportKW) {
            Some(self.required_imports())
        } else {
            None
        }
    }

    pub(crate) fn required_imports(&mut self) -> Result<Imports, ParseError> {
        let import_kw = self.expect(TokenKind::ImportKW)?;
        self.skip_trivia();
        let imports = self.sep(&[TokenKind::Identifier]);
        self.skip_trivia();
        let semicolon = match self.expect(TokenKind::Semicolon) {
            Ok(semicolon) => semicolon,
            Err(e) => match e {
                ParseError::UnexpectedToken { found, .. } => {
                    // expect semicolon or identifier
                    let expected =
                        Expectations::OneOf(vec![TokenKind::Semicolon, TokenKind::Identifier]);
                    return Err(ParseError::UnexpectedToken { found, expected });
                }
                ParseError::UnexpectedEOF { eof, .. } => {
                    // expect semicolon or identifier
                    let expected =
                        Expectations::OneOf(vec![TokenKind::Semicolon, TokenKind::Identifier]);
                    return Err(ParseError::UnexpectedEOF { eof, expected });
                }
            },
        };

        Ok(Imports::new(import_kw, imports, semicolon))
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer, parser::errors::Expectations, Span};

    use super::*;

    #[test]
    fn test_imports() {
        let mut rodeo = Default::default();

        let (tokens, emits) = lexer::lex("IMPORT std foo bar;", 0, &mut rodeo);
        assert!(emits.is_empty());
        let mut parser = Parser::new(&tokens, 0);
        let imports = parser.required_imports().unwrap();

        // Verify token kinds are correct
        assert_eq!(imports.import_kw().kind(), TokenKind::ImportKW);
        assert_eq!(imports.imports().len(), 3);
        assert!(imports
            .imports()
            .iter()
            .all(|t| t.kind() == TokenKind::Identifier));
        assert_eq!(imports.semicolon().kind(), TokenKind::Semicolon);

        // Verify spans are correct
        assert_eq!(imports.span().start(), 0);
        assert_eq!(imports.span().end(), 19);
        assert_eq!(imports.import_kw().span().start(), 0);
        assert_eq!(imports.import_kw().span().end(), 6);
        assert_eq!(imports.imports()[0].span().start(), 7);
        assert_eq!(imports.imports()[0].span().end(), 10);
        assert_eq!(imports.imports()[1].span().start(), 11);
        assert_eq!(imports.imports()[1].span().end(), 14);
        assert_eq!(imports.imports()[2].span().start(), 15);
        assert_eq!(imports.imports()[2].span().end(), 18);
        assert_eq!(imports.semicolon().span().start(), 18);
        assert_eq!(imports.semicolon().span().end(), 19);

        // Verify text is correct
        assert_eq!(imports.import_kw().text(&rodeo), "IMPORT");
        assert_eq!(imports.imports()[0].text(&rodeo), "std");
        assert_eq!(imports.imports()[1].text(&rodeo), "foo");
        assert_eq!(imports.imports()[2].text(&rodeo), "bar");
        assert_eq!(imports.semicolon().text(&rodeo), ";");
    }

    // IMPORT statement requires a semicolon
    #[test]
    fn test_imports_no_semicolon() {
        let mut rodeo = Default::default();

        let text = "IMPORT std foo bar";

        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let err = parser.required_imports().unwrap_err();

        assert_eq!(
            err,
            ParseError::UnexpectedEOF {
                eof: Span::new(text.len(), text.len(), 0),
                expected: Expectations::OneOf(vec![TokenKind::Semicolon, TokenKind::Identifier]),
            }
        );
    }

    // IMPORT statement could be empty
    #[test]
    fn test_imports_no_imports() {
        let mut rodeo = Default::default();

        let text = "IMPORT ;";

        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let err = parser.required_imports().unwrap();

        assert_eq!(err.imports().len(), 0);
    }

    // IMPORT must be made of identifiers
    #[test]
    fn test_imports_invalid_imports() {
        let mut rodeo = Default::default();

        let text = "IMPORT std foo bar 123;";

        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let err = parser.required_imports().unwrap_err();

        assert_eq!(
            err,
            ParseError::UnexpectedToken {
                found: tokens[8].clone(),
                expected: Expectations::OneOf(vec![TokenKind::Identifier, TokenKind::Semicolon]),
            }
        );
    }
}
