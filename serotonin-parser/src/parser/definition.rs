use either::Either;

use crate::{
    ast::{Body, Definition, Quotation},
    Span, Token,
};

use super::{
    errors::{Expectations, ParseError},
    Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        let name = self.expect(Token::Identifier)?;
        self.skip_trivia();
        let stack = match self.optional_stack() {
            Some(s) => Some(s?),
            None => None,
        };
        self.skip_trivia();
        let kind =
            self.expect_one_of(&[Token::Substitution, Token::Generation, Token::Execution])?;
        self.skip_trivia();
        let body = self.parse_body()?;
        self.skip_trivia();
        let semi = self.expect(Token::Semicolon)?;

        Ok(Definition::new(name, stack, kind, body, semi))
    }

    pub(crate) fn parse_quotation(&mut self) -> Result<Quotation, ParseError> {
        let l_bracket = self.expect(Token::LBracket)?;
        self.skip_trivia();
        let body = self.parse_body()?;
        self.skip_trivia();
        let r_bracket = self.expect(Token::RBracket)?;

        Ok(Quotation::new(l_bracket, body, r_bracket))
    }

    pub(crate) fn parse_body(&mut self) -> Result<Body, ParseError> {
        let start = self.index;

        // Track the span of the body
        let mut tokens = Vec::new();

        // While we keep finding tokens
        loop {
            // skip trivia
            self.skip_trivia();

            match self.peek() {
                Some(token) => match token.kind() {
                    Token::LBracket => {
                        tokens.push(Either::Right(self.parse_quotation()?));
                    }
                    Token::RBracket | Token::Semicolon => {
                        break;
                    }
                    t if t.is_atomic() => {
                        tokens.push(Either::Left(self.next().unwrap()));
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            found: token,
                            expected: Expectations::OneOf(Token::atomics().to_vec()),
                        })
                    }
                },
                None => break,
            }
        }
        let span = Span::new(start, self.index, self.file_id);

        Ok(Body::new(span, tokens))
    }
}
