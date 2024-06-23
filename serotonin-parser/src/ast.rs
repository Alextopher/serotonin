//! This module contains a typed Abstract Syntax Tree for the serotonin language
//! 
//! Eventually the AST will be broken out into it's own crate to support the creation of more tools
use lasso::Spur;

use crate::TokenKind;
use crate::{Span, Token};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Module {
    name: Spur,
    imports: Option<Imports>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(name: Spur, imports: Option<Imports>, definitions: Vec<Definition>) -> Self {
        Self {
            name,
            imports,
            definitions,
        }
    }

    /// Returns the modules name
    pub fn name(&self) -> Spur {
        self.name
    }

    /// Returns the modules Imports objects
    pub fn imports(&self) -> Option<&Imports> {
        self.imports.as_ref()
    }

    /// Returns the modules definitions
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Imports {
    import_kw: Token,    // Must be a ImportKW
    imports: Vec<Token>, // Must be Identifiers
    semicolon: Token,
}

impl Imports {
    pub fn new(import_kw: Token, imports: Vec<Token>, semicolon: Token) -> Self {
        debug_assert_eq!(import_kw.kind(), TokenKind::ImportKW);
        debug_assert!(imports.iter().all(|t| t.kind() == TokenKind::Identifier));

        Self {
            import_kw,
            imports,
            semicolon,
        }
    }

    pub fn span(&self) -> Span {
        Span::merge(self.import_kw.span(), self.semicolon.span())
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
            name,
            stack,
            kind,
            body,
            semicolon,
        }
    }

    pub fn span(&self) -> Span {
        Span::merge(self.name.span(), self.semicolon.span())
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
    l_paren: Token, // Must be LParen
    args: Vec<StackArg>,
    r_paren: Token, // Must be RParen
}

impl Stack {
    pub fn new(l_paren: Token, args: Vec<StackArg>, r_paren: Token) -> Self {
        debug_assert_eq!(l_paren.kind(), TokenKind::LParen);
        debug_assert_eq!(r_paren.kind(), TokenKind::RParen);

        Self {
            l_paren,
            r_paren,
            args,
        }
    }

    pub fn span(&self) -> Span {
        Span::merge(self.l_paren.span(), self.r_paren.span())
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
pub enum StackArg {
    UnnamedByte(Token),      // Must be an UnnamedByte
    UnnamedQuotation(Token), // Must be an UnnamedQuotation
    NamedByte(Token),        // Must be a NamedByte
    NamedQuotation(Token),   // Must be a NamedQuotation
    Integer(Token),          // Must be an Integer or HexInteger
    Quotation(Quotation),
}

impl StackArg {
    pub fn span(&self) -> Span {
        match self {
            StackArg::UnnamedByte(token)
            | StackArg::UnnamedQuotation(token)
            | StackArg::NamedByte(token)
            | StackArg::NamedQuotation(token)
            | StackArg::Integer(token) => token.span(),
            StackArg::Quotation(quotation) => quotation.span(),
        }
    }

    pub fn is_quotation(&self) -> bool {
        matches!(self, StackArg::Quotation(_))
    }

    pub fn as_quotation(&self) -> Option<&Quotation> {
        match self {
            StackArg::Quotation(quotation) => Some(quotation),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quotation {
    l_bracket: Token, // Must be LBracket
    body: Body,
    r_bracket: Token, // Must be RBracket
}

impl Quotation {
    pub fn new(l_bracket: Token, body: Body, r_bracket: Token) -> Self {
        debug_assert_eq!(l_bracket.kind(), TokenKind::LBracket);
        debug_assert_eq!(r_bracket.kind(), TokenKind::RBracket);

        Self {
            l_bracket,
            body,
            r_bracket,
        }
    }

    pub fn span(&self) -> Span {
        Span::merge(self.l_bracket().span(), self.r_bracket().span())
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

#[derive(Debug, Clone)]
pub struct Body {
    // The span of a body can not generally be created by merging the spans of its tokens
    // It should span from the first character pass the initializer to the last character before the terminator
    span: Span,
    // Must be an atomic token or a quotation
    tokens: Vec<BodyInner>,
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.tokens == other.tokens
    }
}

impl Eq for Body {}

impl std::hash::Hash for Body {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tokens.hash(state);
    }
}

impl Body {
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
