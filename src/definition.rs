use pest::Span;

#[derive(Debug)]
pub(crate) struct Definition<'a> {
    pub(crate) typ: DefinitionType,
    pub(crate) name: String,
    pub(crate) stack: Option<StackArgs<'a>>,
    pub(crate) body: Vec<Expression<'a>>,
    pub(crate) span: Span<'a>,
    pub(crate) unique_id: usize,
}

impl Definition<'_> {
    pub(crate) fn stack_as_str(&self) -> String {
        match &self.stack {
            Some(s) => s.to_string(),
            None => String::new(),
        }
    }

    pub(crate) fn stack_size(&self) -> usize {
        match &self.stack {
            Some(s) => s.args.len(),
            None => 0,
        }
    }
}

impl std::fmt::Display for Definition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = self
            .body
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        match self.typ {
            DefinitionType::Function => {
                // No stack args
                write!(f, "{} == {}", self.name, body)?;
            }
            DefinitionType::InlineComposition => {
                write!(f, "{} {} == {}", self.name, self.stack_as_str(), body)?
            }
            DefinitionType::ConstantComposition => {
                write!(f, "{} {} ==! {}", self.name, self.stack_as_str(), body)?
            }
            DefinitionType::Composition => {
                write!(f, "{} {} ==? {}", self.name, self.stack_as_str(), body)?
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DefinitionType {
    // Functions compile to normal BF code
    // Results should be cached
    Function,
    // Inline Compositions a simple pattern matching replace.
    // ```
    // swap (a b) == b a;
    // ```
    // means 10 5 swap is replaced with 5 10
    InlineComposition,
    // Constant Compositions pattern match and replace a program with the results of another program
    // ```
    // * (a b) ==! a b * pop;
    // ```
    // 10 20 * is replaced by 200
    ConstantComposition,
    // Compositions are used to build control flow and optimize some functions where applicable
    // For example: read 10 + compiles to `,>++++++++++[-<+>]<` when `,++++++++++` would suffice
    // To create these functions we write programs that _output_ brainfuck as their result
    // ```
    // + (b) ==? '+' b dupn spop;
    // ```
    // 10 + is replaced by `++++++++++`
    Composition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StackArg {
    // Lowercase letter
    Position(char, u8),
    // Capital letter
    Qoutation(char, u8),
    // Number
    Byte(u8),
    // @
    IgnoredConstant,
    // _
    IgnoredQoutation,
}

#[derive(Debug, Clone)]
pub(crate) struct StackArgs<'a> {
    pub(crate) args: Vec<StackArg>,
    pub(crate) span: Span<'a>,
}

// Eq and Hash only consider args and not the pair
impl<'a> PartialEq for StackArgs<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args
    }
}

impl<'a> Eq for StackArgs<'a> {}

impl<'a> std::hash::Hash for StackArgs<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.args.hash(state);
    }
}

impl<'a> std::fmt::Display for StackArgs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for arg in &self.args {
            match arg {
                StackArg::Position(_, n) => {
                    // 0 -> a, 1 -> b, ...
                    write!(f, "{} ", (b'a' + *n) as char)?;
                }
                StackArg::Byte(n) => write!(f, "{} ", n)?,
                StackArg::IgnoredConstant => write!(f, "@ ")?,
                StackArg::Qoutation(_, n) => {
                    // 0 -> A, 1 -> B, ...
                    write!(f, "{} ", (b'A' + *n) as char)?;
                }
                StackArg::IgnoredQoutation => write!(f, "_ ")?,
            }
        }
        write!(f, "\u{8})")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Expression<'a> {
    Constant(u8, Span<'a>),
    Brainfuck(String, Span<'a>),
    Function(String, String, Span<'a>),
    Quotation(Vec<Expression<'a>>, Span<'a>),
}

impl std::fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Do not print the span
        match self {
            Expression::Constant(c, _) => {
                write!(f, "{}", c)?;
            }
            Expression::Quotation(expressions, _) => {
                let body = expressions
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "[{}]", body)?;
            }
            Expression::Brainfuck(bf, _) => {
                write!(f, "`{}`", bf)?;
            }
            Expression::Function(module, name, _) => {
                if module.is_empty() {
                    write!(f, "{}", name)?;
                } else {
                    write!(f, "{}.{}", module, name)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Expression<'a> {
    pub(crate) fn span(&'a self) -> Span<'a> {
        match self {
            Expression::Constant(_, span) => span.clone(),
            Expression::Quotation(_, span) => span.clone(),
            Expression::Brainfuck(_, span) => span.clone(),
            Expression::Function(_, _, span) => span.clone(),
        }
    }

    pub(crate) fn compiled(&self) -> String {
        match self {
            Expression::Constant(n, _) => {
                let mut bf = String::from(">");
                for _ in 0..*n {
                    bf.push('+');
                }
                bf
            }
            Expression::Quotation(expressions, _) => {
                let mut bf = String::new();
                for expression in expressions {
                    bf.push_str(&expression.compiled());
                }
                bf
            }
            Expression::Brainfuck(bf, _) => bf.clone(),
            Expression::Function(..) => {
                panic!("Cannot compile function");
            }
        }
    }
}
