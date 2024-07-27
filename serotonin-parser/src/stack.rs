use crate::{
    ast::{Stack, StackArg},
    Span, TokenKind,
};

use super::{
    errors::{Expectations, ParseError},
    Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn optional_stack(&mut self) -> Option<Result<Stack, ParseError>> {
        if self.peek_is(TokenKind::LParen) {
            Some(self.required_stack())
        } else {
            None
        }
    }

    pub(crate) fn required_stack(&mut self) -> Result<Stack, ParseError> {
        let l_paren = self.expect(TokenKind::LParen)?;
        self.skip_trivia();
        let mut args = Vec::new();
        while !self.peek_is(TokenKind::RParen) {
            args.push(self.parse_stack_arg()?);
            self.skip_trivia();
        }
        let r_paren = self.expect(TokenKind::RParen)?;

        Ok(Stack::new(l_paren, args, r_paren))
    }

    pub(crate) fn parse_stack_arg(&mut self) -> Result<StackArg, ParseError> {
        let expected = Expectations::OneOf(vec![
            TokenKind::LBracket,
            TokenKind::UnnamedByte,
            TokenKind::UnnamedQuotation,
            TokenKind::NamedByte,
            TokenKind::NamedQuotation,
            TokenKind::Integer,
            TokenKind::HexInteger,
        ]);

        // Peek at the next token
        let next = self.peek().ok_or(ParseError::UnexpectedEOF {
            eof: Span::new(self.source_index, self.source_index, self.file_id),
            expected: expected.clone(),
        })?;

        match next.kind() {
            TokenKind::UnnamedByte => Ok(StackArg::UnnamedByte(self.next().unwrap())),
            TokenKind::UnnamedQuotation => Ok(StackArg::UnnamedQuotation(self.next().unwrap())),
            TokenKind::NamedByte => Ok(StackArg::NamedByte(self.next().unwrap())),
            TokenKind::NamedQuotation => Ok(StackArg::NamedQuotation(self.next().unwrap())),
            TokenKind::Integer => Ok(StackArg::Integer(self.next().unwrap())),
            TokenKind::HexInteger => Ok(StackArg::Integer(self.next().unwrap())),
            TokenKind::LBracket => Ok(StackArg::Quotation(self.parse_quotation()?)),
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

    use crate::{
        ast::{Body, BodyInner, Quotation, StackArg},
        Parser, Span, TokenKind,
    };

    #[test]
    fn test_optional_stack() {
        let mut rodeo = Rodeo::default();

        let input = "(a b c)";
        let (tokens, _) = serotonin_lexer::lex(input, 0, &mut rodeo);

        let mut parser = Parser::new(&tokens, 0);
        let stack = parser.required_stack().unwrap();

        assert_eq!(stack.l_paren().kind(), TokenKind::LParen);
        assert_eq!(stack.args().len(), 3);
        assert_eq!(stack.args()[0], StackArg::NamedByte(tokens[1].clone()));
        assert_eq!(stack.args()[1], StackArg::NamedByte(tokens[3].clone()));
        assert_eq!(stack.args()[2], StackArg::NamedByte(tokens[5].clone()));
        assert_eq!(stack.r_paren().kind(), TokenKind::RParen);

        // Verify spans
        assert_eq!(stack.l_paren().span(), Span::new(0, 1, 0));
        assert_eq!(stack.args()[0].span(), Span::new(1, 2, 0));
        assert_eq!(stack.args()[1].span(), Span::new(3, 4, 0));
        assert_eq!(stack.args()[2].span(), Span::new(5, 6, 0));
        assert_eq!(stack.r_paren().span(), Span::new(6, 7, 0));
        assert_eq!(stack.span(), Span::new(0, 7, 0));
    }

    // Test a stack with every type of stack arg
    #[test]
    fn test_stack_args() {
        let mut rodeo = Rodeo::default();

        let input = "(a 0 @ S [true] ?)";
        let (tokens, _) = serotonin_lexer::lex(input, 0, &mut rodeo);

        let mut parser = Parser::new(&tokens, 0);
        let stack = parser.required_stack().unwrap();

        assert_eq!(stack.l_paren().kind(), TokenKind::LParen);
        assert_eq!(stack.args().len(), 6);
        assert_eq!(stack.args()[0], StackArg::NamedByte(tokens[1].clone()));
        assert_eq!(stack.args()[1], StackArg::Integer(tokens[3].clone()));
        assert_eq!(stack.args()[2], StackArg::UnnamedByte(tokens[5].clone()));
        assert_eq!(stack.args()[3], StackArg::NamedQuotation(tokens[7].clone()));
        let quotation = Quotation::new(
            tokens[9].clone(),
            Body::new(
                tokens[10].span(),
                vec![BodyInner::Identifier(tokens[10].clone())],
            ),
            tokens[11].clone(),
        );
        assert_eq!(stack.args()[4].as_quotation().unwrap(), &quotation);
        assert_eq!(
            stack.args()[5],
            StackArg::UnnamedQuotation(tokens[13].clone())
        );
    }
}
