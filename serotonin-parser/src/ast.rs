/// This module contains a typed Abstract Syntax Tree for the serotonin language
use lasso::Spur;

use crate::{Span, Token};
use crate::TokenKind;

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Imports {
    span: Span,
    import_kw: Token,    // Must be a ImportKW
    imports: Vec<Token>, // Must be Identifiers
    semicolon: Token,
}

impl Imports {
    pub fn new(import_kw: Token, imports: Vec<Token>, semicolon: Token) -> Self {
        debug_assert_eq!(import_kw.kind(), TokenKind::ImportKW);
        debug_assert!(imports.iter().all(|t| t.kind() == TokenKind::Identifier));

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

    pub fn import_kw(&self) -> Token {
        self.import_kw.clone()
    }

    pub fn imports(&self) -> &[Token] {
        &self.imports
    }

    pub fn semicolon(&self) -> Token {
        self.semicolon.clone()
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Definition {
    span: Span,
    name: Token, // Must be an identifier
    stack: Option<Stack>,
    kind: Token, // Must be Substitution, Generation, or Execution
    body: Body,
    semicolon: Token, // Must be a Semicolon
}

impl Definition {
    pub fn new(
        name: Token,
        stack: Option<Stack>,
        kind: Token,
        body: Body,
        semicolon: Token,
    ) -> Self {
        // name must be an identifier
        debug_assert_eq!(name.kind(), TokenKind::Identifier);
        // kind must be Substitution, Generation, or Execution
        debug_assert!(matches!(
            kind.kind(),
            TokenKind::Substitution | TokenKind::Generation | TokenKind::Execution
        ));
        // semi must be a Semicolon
        debug_assert_eq!(semicolon.kind(), TokenKind::Semicolon);

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

    pub fn name(&self) -> Token {
        self.name.clone()
    }

    pub fn stack(&self) -> Option<&Stack> {
        self.stack.as_ref()
    }

    pub fn kind(&self) -> Token {
        self.kind.clone()
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn semicolon(&self) -> Token {
        self.semicolon.clone()
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stack {
    span: Span,
    l_paren: Token, // Must be LParen
    args: Vec<StackArg>,
    r_paren: Token, // Must be RParen
}

impl Stack {
    pub fn new(l_paren: Token, args: Vec<StackArg>, r_paren: Token) -> Self {
        debug_assert_eq!(l_paren.kind(), TokenKind::LParen);
        debug_assert_eq!(r_paren.kind(), TokenKind::RParen);

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

    pub fn l_paren(&self) -> Token {
        self.l_paren.clone()
    }

    pub fn args(&self) -> &[StackArg] {
        &self.args
    }

    pub fn r_paren(&self) -> Token {
        self.r_paren.clone()
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackArg {
    span: Span,
    inner: StackArgInner,
}

impl StackArg {
    pub fn new(inner: StackArgInner) -> Self {
        Self {
            span: inner.span(),
            inner,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn inner(&self) -> &StackArgInner {
        &self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackArgInner {
    UnnamedByte(Token),      // Must be an UnnamedByte
    UnnamedQuotation(Token), // Must be an UnnamedQuotation
    NamedByte(Token),        // Must be a NamedByte
    NamedQuotation(Token),   // Must be a NamedQuotation
    Integer(Token),          // Must be an Integer or HexInteger
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
    l_bracket: Token, // Must be LBracket
    body: Body,
    r_bracket: Token, // Must be RBracket
}

impl Quotation {
    pub fn new(l_bracket: Token, body: Body, r_bracket: Token) -> Self {
        debug_assert_eq!(l_bracket.kind(), TokenKind::LBracket);
        debug_assert_eq!(r_bracket.kind(), TokenKind::RBracket);

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

    pub fn l_bracket(&self) -> Token {
        self.l_bracket.clone()
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn r_bracket(&self) -> Token {
        self.r_bracket.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Body {
    span: Span,
    // Must be an atomic token or a quotation
    tokens: Vec<BodyInner>,
}

impl Body {
    // The span of the body should not include brackets / other terminators
    // It can not generally be created by merging the spans of its tokens
    pub fn new(span: Span, tokens: Vec<BodyInner>) -> Self {
        Self { span, tokens }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn tokens(&self) -> &[BodyInner] {
        &self.tokens
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BodyInner {
    // Atomics
    Integer(Token),
    HexInteger(Token),
    String(Token),
    RawString(Token),
    MacroInput(Token),
    NamedByte(Token),
    NamedQuotation(Token),
    Identifier(Token),
    Brainfuck(Token),
    // Quotation
    Quotation(Quotation),
    // Identifier Dot Identifier.
    FQN(FQN),
}

impl BodyInner {
    /// Construct a InternedToken
    // pub fn 

    pub fn span(&self) -> Span {
        match self {
            BodyInner::Integer(token)
            | BodyInner::HexInteger(token)
            | BodyInner::String(token)
            | BodyInner::RawString(token)
            | BodyInner::MacroInput(token)
            | BodyInner::NamedByte(token)
            | BodyInner::NamedQuotation(token)
            | BodyInner::Identifier(token)
            | BodyInner::Brainfuck(token) => token.span(),
            BodyInner::Quotation(quotation) => quotation.span(),
            BodyInner::FQN(FQN { module, name, .. }) => Span::merge(module.span(), name.span()),
        }
    }

    /// If self is storing a `InternedToken` return it
    pub fn token(&self) -> Option<Token> {
        match self {
            BodyInner::Integer(token)
            | BodyInner::HexInteger(token)
            | BodyInner::String(token)
            | BodyInner::RawString(token)
            | BodyInner::MacroInput(token)
            | BodyInner::NamedByte(token)
            | BodyInner::NamedQuotation(token)
            | BodyInner::Identifier(token)
            | BodyInner::Brainfuck(token) => Some(token.clone()),
            _ => None,
        }
    }

    pub fn quotation(&self) -> Option<&Quotation> {
        match self {
            BodyInner::Quotation(quotation) => Some(quotation),
            _ => None,
        }
    }

    pub fn fqn(&self) -> Option<&FQN> {
        match self {
            BodyInner::FQN(fqn) => Some(fqn),
            _ => None,
        }
    }
}

/// Fully qualified name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FQN {
    module: Token,
    dot: Token,
    name: Token,
}

impl FQN {
    pub fn new(module: Token, dot: Token, name: Token) -> Self {
        debug_assert!(module.kind() == TokenKind::Identifier);
        debug_assert!(dot.kind() == TokenKind::Dot);
        debug_assert!(name.kind() == TokenKind::Identifier);

        Self { module, dot, name }
    }
}
