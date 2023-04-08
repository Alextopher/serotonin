/// This module contains a typed Abstract Syntax Tree for the serotonin language
use std::fmt::Write;

use either::Either;
use lasso::{Rodeo, Spur};

use crate::Span;
use crate::{Token, TokenType};

pub(crate) trait Print {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    span: Span,
    name: Spur,
    imports: Option<Imports>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        name: Spur,
        imports: Option<Imports>,
        definitions: Vec<Definition>,
        span: Span,
    ) -> Self {
        Self {
            span,
            name,
            imports,
            definitions,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> Spur {
        self.name
    }

    pub fn imports(&self) -> Option<&Imports> {
        self.imports.as_ref()
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}

impl Print for Module {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        writeln!(f, "Module: {}", rodeo.resolve(&self.name))?;

        for definition in &self.definitions {
            definition.print(f, rodeo)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Imports {
    span: Span,
    import_kw: TokenType,    // Must be a ImportKW
    imports: Vec<TokenType>, // Must be Identifiers
    semicolon: TokenType,
}

impl Imports {
    pub fn new(import_kw: TokenType, imports: Vec<TokenType>, semicolon: TokenType) -> Self {
        debug_assert_eq!(import_kw.kind(), Token::ImportKW);
        debug_assert!(imports.iter().all(|t| t.kind() == Token::Identifier));

        Self {
            span: Span::merge(import_kw.span(), semicolon.span()),
            import_kw,
            imports,
            semicolon,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn import_kw(&self) -> TokenType {
        self.import_kw.clone()
    }

    pub fn imports(&self) -> &[TokenType] {
        &self.imports
    }

    pub fn semicolon(&self) -> TokenType {
        self.semicolon.clone()
    }
}

impl Print for Imports {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        write!(f, "IMPORT ")?;
        for (i, import) in self.imports.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", rodeo.resolve(&import.spur()))?;
        }
        write!(f, ";")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    span: Span,
    name: TokenType, // Must be an identifier
    stack: Option<Stack>,
    kind: TokenType, // Must be Substitution, Generation, or Execution
    body: Body,
    semicolon: TokenType, // Must be a Semicolon
}

impl Definition {
    pub fn new(
        name: TokenType,
        stack: Option<Stack>,
        kind: TokenType,
        body: Body,
        semicolon: TokenType,
    ) -> Self {
        // name must be an identifier
        debug_assert_eq!(name.kind(), Token::Identifier);
        // kind must be Substitution, Generation, or Execution
        debug_assert!(matches!(
            kind.kind(),
            Token::Substitution | Token::Generation | Token::Execution
        ));
        // semi must be a Semicolon
        debug_assert_eq!(semicolon.kind(), Token::Semicolon);

        Self {
            span: Span::merge(name.span(), semicolon.span()),
            name,
            stack,
            kind,
            body,
            semicolon,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &TokenType {
        &self.name
    }

    pub fn stack(&self) -> Option<&Stack> {
        self.stack.as_ref()
    }

    pub fn kind(&self) -> &TokenType {
        &self.kind
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn semicolon(&self) -> &TokenType {
        &self.semicolon
    }
}

impl Print for Definition {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        write!(f, "{} ", rodeo.resolve(&self.name.spur()))?;
        if let Some(stack) = &self.stack {
            stack.print(f, rodeo)?;
            write!(f, " ")?;
        }
        write!(f, "{} ", rodeo.resolve(&self.kind.spur()))?;
        self.body.print(f, rodeo)?;
        writeln!(f, ";")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stack {
    span: Span,
    l_paren: TokenType, // Must be LParen
    args: Vec<StackArg>,
    r_paren: TokenType, // Must be RParen
}

impl Stack {
    pub fn new(l_paren: TokenType, args: Vec<StackArg>, r_paren: TokenType) -> Self {
        debug_assert_eq!(l_paren.kind(), Token::LParen);
        debug_assert_eq!(r_paren.kind(), Token::RParen);

        Self {
            span: Span::merge(l_paren.span(), r_paren.span()),
            l_paren,
            r_paren,
            args,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn l_paren(&self) -> &TokenType {
        &self.l_paren
    }

    pub fn args(&self) -> &[StackArg] {
        &self.args
    }

    pub fn r_paren(&self) -> &TokenType {
        &self.r_paren
    }
}

impl Print for Stack {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            arg.print(f, rodeo)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackArg {
    span: Span,
    inner: StackArgInner,
}

impl StackArg {
    pub fn new(inner: Either<TokenType, Quotation>) -> Self {
        match inner {
            Either::Left(token) => {
                debug_assert!(matches!(
                    token.kind(),
                    Token::UnnamedByte
                        | Token::UnnamedQuotation
                        | Token::NamedByte
                        | Token::NamedQuotation
                        | Token::Integer
                        | Token::HexInteger
                ));

                StackArg {
                    span: token.span(),
                    inner: match token.kind() {
                        Token::UnnamedByte => StackArgInner::UnnamedByte(token),
                        Token::UnnamedQuotation => StackArgInner::UnnamedQuotation(token),
                        Token::NamedByte => StackArgInner::NamedByte(token),
                        Token::NamedQuotation => StackArgInner::NamedQuotation(token),
                        Token::Integer | Token::HexInteger => StackArgInner::Integer(token),
                        _ => unreachable!(),
                    },
                }
            }
            Either::Right(quotation) => StackArg {
                span: quotation.span,
                inner: StackArgInner::Quotation(quotation),
            },
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn inner(&self) -> &StackArgInner {
        &self.inner
    }

    pub fn text(&self, rodeo: &Rodeo) -> String {
        match &self.inner {
            StackArgInner::UnnamedByte(token)
            | StackArgInner::UnnamedQuotation(token)
            | StackArgInner::NamedByte(token)
            | StackArgInner::NamedQuotation(token)
            | StackArgInner::Integer(token) => rodeo.resolve(&token.spur()).to_string(),
            StackArgInner::Quotation(quotation) => quotation.text(rodeo),
        }
    }
}

impl Print for StackArg {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        match &self.inner {
            StackArgInner::UnnamedByte(token)
            | StackArgInner::UnnamedQuotation(token)
            | StackArgInner::NamedByte(token)
            | StackArgInner::NamedQuotation(token)
            | StackArgInner::Integer(token) => write!(f, "{}", rodeo.resolve(&token.spur())),
            StackArgInner::Quotation(quotation) => quotation.print(f, rodeo),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackArgInner {
    UnnamedByte(TokenType),      // Must be an UnnamedByte
    UnnamedQuotation(TokenType), // Must be an UnnamedQuotation
    NamedByte(TokenType),        // Must be a NamedByte
    NamedQuotation(TokenType),   // Must be a NamedQuotation
    Integer(TokenType),          // Must be an Integer or HexInteger
    Quotation(Quotation),
}

impl From<StackArgInner> for StackArg {
    fn from(val: StackArgInner) -> Self {
        StackArg {
            span: val.span(),
            inner: val,
        }
    }
}

impl StackArgInner {
    pub fn span(&self) -> Span {
        match self {
            StackArgInner::UnnamedByte(token)
            | StackArgInner::UnnamedQuotation(token)
            | StackArgInner::NamedByte(token)
            | StackArgInner::NamedQuotation(token)
            | StackArgInner::Integer(token) => token.span(),
            StackArgInner::Quotation(quotation) => quotation.span,
        }
    }

    pub fn is_quotation(&self) -> bool {
        matches!(self, StackArgInner::Quotation(_))
    }

    pub fn as_quotation(&self) -> Option<&Quotation> {
        match self {
            StackArgInner::Quotation(quotation) => Some(quotation),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quotation {
    span: Span,
    l_bracket: TokenType, // Must be LBracket
    body: Body,
    r_bracket: TokenType, // Must be RBracket
}

impl Quotation {
    pub fn new(l_bracket: TokenType, body: Body, r_bracket: TokenType) -> Self {
        debug_assert_eq!(l_bracket.kind(), Token::LBracket);
        debug_assert_eq!(r_bracket.kind(), Token::RBracket);

        Self {
            span: Span::merge(l_bracket.span(), r_bracket.span()),
            l_bracket,
            body,
            r_bracket,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn l_bracket(&self) -> &TokenType {
        &self.l_bracket
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn r_bracket(&self) -> &TokenType {
        &self.r_bracket
    }

    pub fn text(&self, rodeo: &Rodeo) -> String {
        let mut f = String::new();
        self.print(&mut f, rodeo).unwrap();
        f
    }
}

impl Print for Quotation {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, token) in self.body.tokens.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            match token {
                Either::Left(token) => write!(f, "{}", rodeo.resolve(&token.spur()))?,
                Either::Right(quotation) => quotation.print(f, rodeo)?,
            }
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Body {
    span: Span,
    // Must be an atomic token or a quotation
    tokens: Vec<Either<TokenType, Quotation>>,
}

impl Body {
    pub fn new(span: Span, tokens: Vec<Either<TokenType, Quotation>>) -> Self {
        debug_assert!(tokens.iter().all(|t| match t {
            Either::Left(token) => token.kind().is_atomic(),
            Either::Right(_) => true,
        }));

        Self { span, tokens }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn tokens(&self) -> &[Either<TokenType, Quotation>] {
        &self.tokens
    }
}

impl Print for Body {
    fn print(&self, f: &mut String, rodeo: &Rodeo) -> std::fmt::Result {
        for (i, token) in self.tokens.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            match token {
                Either::Left(token) => write!(f, "{}", rodeo.resolve(&token.spur()))?,
                Either::Right(quotation) => quotation.print(f, rodeo)?,
            }
        }
        Ok(())
    }
}
