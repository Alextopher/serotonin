use either::Either;

use crate::{
    ast::{Stack, StackArg},
    Span, Token,
};

use super::{
    errors::{Expectations, ParseError},
    Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn optional_stack(&mut self) -> Option<Result<Stack, ParseError>> {
        if self.peek_is(Token::LParen) {
            Some(self.required_stack())
        } else {
            None
        }
    }

    pub(crate) fn required_stack(&mut self) -> Result<Stack, ParseError> {
        let l_paren = self.expect(Token::LParen)?;
        self.skip_trivia();
        let mut args = Vec::new();
        while !self.peek_is(Token::RParen) {
            args.push(self.parse_stack_arg()?);
            self.skip_trivia();
        }
        let r_paren = self.expect(Token::RParen)?;

        Ok(Stack::new(l_paren, args, r_paren))
    }

    pub(crate) fn parse_stack_arg(&mut self) -> Result<StackArg, ParseError> {
        let expected = Expectations::OneOf(vec![
            Token::RBracket,
            Token::UnnamedByte,
            Token::UnnamedQuotation,
            Token::NamedByte,
            Token::NamedQuotation,
            Token::Integer,
            Token::HexInteger,
        ]);

        // Peek at the next token
        let next = self.next().ok_or(ParseError::UnexpectedEOF {
            eof: Span::new(self.index, self.index, self.file_id),
            expected: expected.clone(),
        })?;

        match next.kind() {
            Token::UnnamedByte
            | Token::UnnamedQuotation
            | Token::NamedByte
            | Token::NamedQuotation
            | Token::Integer
            | Token::HexInteger => Ok(StackArg::new(Either::Left(next))),
            Token::RBracket => {
                let quotation = self.parse_quotation()?;
                Ok(StackArg::new(Either::Right(quotation)))
            }
            _ => Err(ParseError::UnexpectedToken {
                found: next,
                expected,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use lasso::Rodeo;

    use crate::{ast::StackArgInner, lexer, parser::Parser, Span, Token};

    #[test]
    fn test_optional_stack() {
        let mut rodeo = Rodeo::default();

        let input = "(a b c)";
        let (tokens, _) = lexer::lex(input, 0, &mut rodeo);

        let mut parser = Parser::new(&tokens, 0);
        let stack = parser.required_stack().unwrap();

        assert_eq!(stack.l_paren().kind(), Token::LParen);
        assert_eq!(stack.args().len(), 3);
        assert_eq!(
            stack.args()[0].inner(),
            &StackArgInner::NamedByte(tokens[1].clone())
        );
        assert_eq!(
            stack.args()[1].inner(),
            &StackArgInner::NamedByte(tokens[3].clone())
        );
        assert_eq!(
            stack.args()[2].inner(),
            &StackArgInner::NamedByte(tokens[5].clone())
        );
        assert_eq!(stack.r_paren().kind(), Token::RParen);

        // Verify spans
        assert_eq!(stack.l_paren().span(), Span::new(0, 1, 0));
        assert_eq!(stack.args()[0].span(), Span::new(1, 2, 0));
        assert_eq!(stack.args()[1].span(), Span::new(3, 4, 0));
        assert_eq!(stack.args()[2].span(), Span::new(5, 6, 0));
        assert_eq!(stack.r_paren().span(), Span::new(6, 7, 0));
        assert_eq!(stack.span(), Span::new(0, 7, 0));

        // Verify text
        assert_eq!(stack.l_paren().text(&rodeo), "(");
        assert_eq!(stack.r_paren().text(&rodeo), ")");
        assert_eq!(stack.args()[0].text(&rodeo), "a");
    }
}
