use crate::{
    ast::{Body, BodyInner, Definition, Quotation, FQN},
    Span, TokenKind,
};

use super::{
    errors::{Expectations, ParseError},
    Parser,
};

impl Parser<'_> {
    pub(crate) fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        let name = self.expect(TokenKind::Identifier)?;
        self.skip_trivia();
        let stack = self.optional_stack().transpose()?;
        self.skip_trivia();
        let kind = self.expect_one_of(&[
            TokenKind::Substitution,
            TokenKind::Generation,
            TokenKind::Execution,
        ])?;
        self.skip_trivia();
        let body = self.parse_body(TokenKind::Semicolon)?;
        self.skip_trivia();
        let semi = self.expect(TokenKind::Semicolon)?;

        Ok(Definition::new(name, stack, kind, body, semi))
    }

    /// Parses a quotation (e.g. `[1 2 3 ]`)
    /// The trivia within the body is handled by `parse_body`
    pub(crate) fn parse_quotation(&mut self) -> Result<Quotation, ParseError> {
        let l_bracket = self.expect(TokenKind::LBracket)?;
        let body = self.parse_body(TokenKind::RBracket)?;
        let r_bracket = self.expect(TokenKind::RBracket)?;

        Ok(Quotation::new(l_bracket, body, r_bracket))
    }

    /// Repeatedly parses body tokens until the given token is found
    /// This span of the body is all characters between the opening token and the closing token
    ///
    /// For example:
    /// [  a b c]
    ///  ^^^^^^^
    pub(crate) fn parse_body(&mut self, until: TokenKind) -> Result<Body, ParseError> {
        // We only expect to stop on a closing bracket or a semicolon, depending if we
        // are parsing a quotation or a definition
        debug_assert!(until == TokenKind::RBracket || until == TokenKind::Semicolon);

        let start = self.source_index;

        let mut body = Vec::new();
        while !self.peek_is(until) {
            body.push(self.parse_body_inner()?);
            self.skip_trivia();
        }

        let end = self.source_index;

        Ok(Body::new(Span::new(start, end, self.file_id), body))
    }

    // Either
    // - atomic token (identifier, integer, etc)
    // - a quotation
    // - Fully Qualified name with no whitespace between (id.id)
    pub(crate) fn parse_body_inner(&mut self) -> Result<BodyInner, ParseError> {
        let expected = Expectations::OneOf(
            [
                TokenKind::Integer,
                TokenKind::HexInteger,
                TokenKind::String,
                TokenKind::RawString,
                TokenKind::MacroInput,
                TokenKind::NamedByte,
                TokenKind::NamedQuotation,
                TokenKind::Brainfuck,
                TokenKind::Identifier,
                TokenKind::LBracket,
            ]
            .to_vec(),
        );

        match self.peek() {
            Some(next) => {
                match next.kind() {
                    // Atomics
                    TokenKind::Integer => Ok(BodyInner::Integer(self.next().unwrap())),
                    TokenKind::HexInteger => Ok(BodyInner::HexInteger(self.next().unwrap())),
                    TokenKind::String => Ok(BodyInner::String(self.next().unwrap())),
                    TokenKind::RawString => Ok(BodyInner::RawString(self.next().unwrap())),
                    TokenKind::MacroInput => Ok(BodyInner::MacroInput(self.next().unwrap())),
                    TokenKind::NamedByte => Ok(BodyInner::NamedByte(self.next().unwrap())),
                    TokenKind::NamedQuotation => {
                        Ok(BodyInner::NamedQuotation(self.next().unwrap()))
                    }
                    TokenKind::Brainfuck => Ok(BodyInner::Brainfuck(self.next().unwrap())),
                    // Identifier either starts a FQN or is an atomic
                    TokenKind::Identifier => {
                        // Identifier could be the start of a fully qualified name
                        // or it could be a single identifier
                        let first = self.expect(TokenKind::Identifier).unwrap(); // Safe to unwrap because we know it's an identifier

                        if self.peek_is(TokenKind::Dot) {
                            // Fully qualified name
                            let dot = self.expect(TokenKind::Dot).unwrap(); // Safe to unwrap because we know it's a dot

                            // Now we expect to have another identifier
                            match self.expect(TokenKind::Identifier) {
                                Ok(second) => Ok(BodyInner::FQN(FQN::new(first, dot, second))),
                                Err(e) => Err(e),
                            }
                        } else {
                            // Single identifier
                            Ok(BodyInner::Identifier(first))
                        }
                    }
                    TokenKind::LBracket => self.parse_quotation().map(BodyInner::Quotation),
                    _ => Err(ParseError::UnexpectedToken {
                        found: next,
                        expected,
                    }),
                }
            }
            None => Err(ParseError::UnexpectedEOF {
                eof: Span::new(self.source_index, self.source_index, self.file_id),
                expected,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{BodyInner, Stack, StackArg, StackArgInner},
        lexer,
        parser::Parser, TokenKind,
    };

    #[test]
    fn simple_quotation() {
        let mut rodeo = Default::default();

        let text = "[1 2 3 ]";

        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let quotation = parser.parse_quotation().unwrap();

        assert_eq!(quotation.r_bracket().kind(), TokenKind::RBracket);
        assert_eq!(quotation.l_bracket().kind(), TokenKind::LBracket);
        assert_eq!(quotation.body().tokens().len(), 3);
        assert_eq!(
            quotation.body().tokens()[0],
            BodyInner::Integer(tokens[1].clone())
        );
        assert_eq!(
            quotation.body().tokens()[1],
            BodyInner::Integer(tokens[3].clone())
        );
        assert_eq!(
            quotation.body().tokens()[2],
            BodyInner::Integer(tokens[5].clone())
        );

        assert_eq!(quotation.span().start(), 0);
        assert_eq!(quotation.span().end(), 8);

        assert_eq!(quotation.body().span().start(), 1);
        assert_eq!(quotation.body().span().end(), 7); // 7 because we include the whitespace but not the closing bracket
    }

    // Nested quotation with depth 3
    #[test]
    fn nested_quotation() {
        let mut rodeo = Default::default();

        let text = "[[1 2 3] [4 5 6] [7 8 9]]";

        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let quotation = parser.parse_quotation().unwrap();

        assert_eq!(quotation.r_bracket().kind(), TokenKind::RBracket);
        assert_eq!(quotation.l_bracket().kind(), TokenKind::LBracket);
        assert_eq!(quotation.body().tokens().len(), 3);

        let first = quotation.body().tokens()[0].quotation().unwrap();
        assert_eq!(first.r_bracket().kind(), TokenKind::RBracket);
        assert_eq!(first.l_bracket().kind(), TokenKind::LBracket);
        assert_eq!(first.body().tokens().len(), 3);
        assert_eq!(
            first.body().tokens()[0],
            BodyInner::Integer(tokens[2].clone())
        );
        assert_eq!(
            first.body().tokens()[1],
            BodyInner::Integer(tokens[4].clone())
        );
        assert_eq!(
            first.body().tokens()[2],
            BodyInner::Integer(tokens[6].clone())
        );

        let second = quotation.body().tokens()[1].quotation().unwrap();
        assert_eq!(second.r_bracket().kind(), TokenKind::RBracket);
        assert_eq!(second.l_bracket().kind(), TokenKind::LBracket);
        assert_eq!(second.body().tokens().len(), 3);
        assert_eq!(
            second.body().tokens()[0],
            BodyInner::Integer(tokens[10].clone())
        );
        assert_eq!(
            second.body().tokens()[1],
            BodyInner::Integer(tokens[12].clone())
        );
        assert_eq!(
            second.body().tokens()[2],
            BodyInner::Integer(tokens[14].clone())
        );

        let third = quotation.body().tokens()[2].quotation().unwrap();
        assert_eq!(third.r_bracket().kind(), TokenKind::RBracket);
        assert_eq!(third.l_bracket().kind(), TokenKind::LBracket);
        assert_eq!(third.body().tokens().len(), 3);
        assert_eq!(
            third.body().tokens()[0],
            BodyInner::Integer(tokens[18].clone())
        );
        assert_eq!(
            third.body().tokens()[1],
            BodyInner::Integer(tokens[20].clone())
        );
        assert_eq!(
            third.body().tokens()[2],
            BodyInner::Integer(tokens[22].clone())
        );

        assert_eq!(quotation.span().start(), 0);
        assert_eq!(quotation.span().end(), 25);

        assert_eq!(quotation.body().span().start(), 1);
        assert_eq!(quotation.body().span().end(), 24);
    }

    // A full definition
    #[test]
    fn definition() {
        let mut rodeo = Default::default();

        let text = "dup (a) == a a;";
        let (tokens, emits) = lexer::lex(text, 0, &mut rodeo);
        assert!(emits.is_empty());

        let mut parser = Parser::new(&tokens, 0);
        let definition = parser.parse_definition().unwrap();

        assert_eq!(definition.name().kind(), TokenKind::Identifier);
        assert_eq!(definition.name().span().start(), 0);
        assert_eq!(definition.name().span().end(), 3);

        let expected_stack = Stack::new(
            tokens[2].clone(),
            vec![StackArg::new(StackArgInner::NamedByte(tokens[3].clone()))],
            tokens[4].clone(),
        );

        assert_eq!(definition.stack().unwrap(), &expected_stack);

        assert_eq!(definition.kind().kind(), TokenKind::Substitution);
        assert_eq!(definition.kind().span().start(), 8);
        assert_eq!(definition.kind().span().end(), 10);

        assert_eq!(definition.body().tokens().len(), 2);
        assert_eq!(
            definition.body().tokens()[0],
            BodyInner::NamedByte(tokens[8].clone())
        );
        assert_eq!(
            definition.body().tokens()[1],
            BodyInner::NamedByte(tokens[10].clone())
        );
    }
}
