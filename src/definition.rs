use std::{slice::from_raw_parts, sync::Arc};

#[derive(Clone, Ord, Hash, Eq, PartialOrd, PartialEq)]
pub struct Span {
    pub(crate) data: Arc<String>,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<'a, 'b: 'a> From<&'b Span> for pest::Span<'a> {
    fn from(other: &'b Span) -> pest::Span<'a> {
        pest::Span::new(&other.data, other.start, other.end).unwrap()
    }
}

impl<'a> From<pest::Span<'a>> for Span {
    fn from(other: pest::Span<'a>) -> Span {
        // SAFETY:
        // In pest::Span the `input` field is hidden away in a private field and this is fighting to get it out.
        // `ptr` is valid because pest::Span ensures that `input[start]` is within the bounds of `input`
        // `s` is good because we know `ptr` was created from `input` which was a valid &str
        let ptr = unsafe { 
            let ptr = other.as_str().as_ptr() as *const u8;
            ptr.sub(other.start())
        };
        let s = unsafe { std::str::from_utf8_unchecked(from_raw_parts(ptr, other.end() + 1)) };

        Span {
            data: Arc::new(s.to_string()),
            start: other.start(),
            end: other.end(),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.data[self.start..self.end])
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indexed = &self.data[self.start..self.end];

        f.debug_struct("Span")
            .field("data", &indexed)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct Definition {
    pub(crate) typ: DefinitionType,
    pub(crate) name: String,
    pub(crate) stack: Option<StackArgs>,
    pub(crate) body: Vec<Expression>,
    pub(crate) span: Span,
    pub(crate) unique_id: usize,
}

impl Definition {
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

impl std::fmt::Display for Definition {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct StackArgs {
    pub(crate) args: Vec<StackArg>,
}

impl std::fmt::Display for StackArgs {
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
pub(crate) enum Expression {
    Constant(u8, Span),
    Brainfuck(String, Span),
    Function(String, String, Span),
    Quotation(Vec<Expression>, Span),
    Macro(String, String, Span),
}

impl std::fmt::Display for Expression {
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
            },
            Expression::Macro(input, method, _) => {
                write!(f, "{{{}}} {}", input, method)?;
            }
        }

        Ok(())
    }
}

impl Expression {
    pub(crate) fn span(&self) -> &Span {
        match self {
            Expression::Constant(_, span) => span,
            Expression::Quotation(_, span) => span,
            Expression::Brainfuck(_, span) => span,
            Expression::Function(_, _, span) => span,
            Expression::Macro(_, _, span) => span,
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
            },
            Expression::Macro(_, _, _) => {
                panic!("Cannot compile macros here")
            }
        }
    }
}
