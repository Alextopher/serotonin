use crate::{pest::Parser, semantic::apply_semantics, stdlib::LIBRARIES};
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
    Span,
};
use pest_derive::Parser;
use std::fmt::Write;
use std::{collections::HashMap, iter};

#[derive(Parser)]
#[grammar = "serotonin.pest"]
struct PestParser;

pub fn compile(name: &str, input: &str) -> Result<String, Vec<Error<Rule>>> {
    // Time sections
    let mut start = std::time::Instant::now();

    let pairs = match PestParser::parse(Rule::module, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(vec![e]),
    };

    println!("Pest parsing took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let this = parse_module_ast(name.to_owned(), pairs)?;

    println!("Parsing took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let mut asts = Dependencies::resolve(&this)?;
    asts.insert(name, this);
    // println!(
    //     "{}",
    //     asts.get(name).unwrap().definitions.get("main").unwrap()[0]
    // );

    println!("Parsing imports took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let new_ast = apply_semantics(&mut asts)?;
    // println!(
    //     "{}",
    //     new_ast.get(name).unwrap().definitions.get("main").unwrap()[0]
    // );

    println!("Checking semantics took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    Ok(String::new())
}

pub(crate) struct Dependencies<'a> {
    building: Vec<&'a str>,
    asts: HashMap<&'a str, ModuleAst<'a>>,
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
    pub(crate) pair: Pair<'a, Rule>,
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

#[derive(Debug)]
pub(crate) struct Definition<'a> {
    pub(crate) typ: DefinitionType,
    pub(crate) name: String,
    pub(crate) stack: Option<StackArgs<'a>>,
    pub(crate) body: Vec<Expression<'a>>,
    pub(crate) pair: Pair<'a, Rule>,
}

impl Definition<'_> {
    pub(crate) fn stack_as_str(&self) -> String {
        match &self.stack {
            Some(s) => s.to_string(),
            None => String::new(),
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

#[derive(Debug, Clone)]
pub(crate) enum Expression<'a> {
    Constant(u8, Span<'a>),
    Brainfuck(String, Span<'a>),
    Function {
        module: String,
        name: String,
        span: Span<'a>,
    },
    Quotation(Vec<Expression<'a>>, Span<'a>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Expression<'a> {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::atomic => {
                // Either "module.function" or "function"
                if let Some(dot) = pair.as_str().find('.') {
                    let (module, name) = pair.as_str().split_at(dot);

                    Ok(Expression::Function {
                        module: String::from(module),
                        name: String::from(&name[1..]),
                        span: pair.as_span(),
                    })
                } else {
                    Ok(Expression::Function {
                        module: String::new(),
                        name: pair.as_str().to_string(),
                        span: pair.as_span(),
                    })
                }
            }
            Rule::hex_integer => match u8::from_str_radix(&pair.as_str()[2..], 16) {
                Ok(byte) => Ok(Expression::Constant(byte, pair.as_span())),
                Err(err) => Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("{}", err),
                    },
                    pair.as_span(),
                )),
            },
            Rule::integer => match pair.as_str().parse::<u8>() {
                Ok(byte) => Ok(Expression::Constant(byte, pair.as_span())),
                Err(err) => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("{}", err),
                        },
                        pair.as_span(),
                    ))
                }
            },
            Rule::string => {
                let mut constants =
                    vec![Expression::Constant(0, pair.as_span().get(0..1).unwrap())];
                constants.extend(pair.clone().into_inner().map(constant_from_char));

                Ok(Expression::Quotation(constants, pair.clone().as_span()))
            }
            Rule::raw_string => Ok(Expression::Quotation(
                pair.clone().into_inner().map(constant_from_char).collect(),
                pair.as_span(),
            )),
            Rule::brainfuck => Ok(Expression::Brainfuck(
                pair.as_str().to_string(),
                pair.as_span(),
            )),
            Rule::term => Ok(Expression::Quotation(
                parse_term_ast(pair.clone())?,
                pair.as_span(),
            )),
            _ => unreachable!(),
        }
    }
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
                write!(f, "{}", bf)?;
            }
            Expression::Function {
                module,
                name,
                span: _,
            } => {
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

impl<'a> Dependencies<'a> {
    fn resolve(main: &ModuleAst<'a>) -> Result<HashMap<&'a str, ModuleAst<'a>>, Vec<Error<Rule>>> {
        let mut dep = Dependencies {
            building: Vec::new(),
            asts: HashMap::new(),
        };

        for import in main.imports.clone() {
            if let Err(e) = dep._resolve(import) {
                return Err(e);
            }
        }

        Ok(dep.asts)
    }

    fn _resolve(&mut self, module: Pair<'a, Rule>) -> Result<(), Vec<Error<Rule>>> {
        assert_eq!(module.as_rule(), Rule::atomic);

        if self.building.contains(&module.as_str()) {
            // build the cycle so it can be reported
            let mut cycle = Vec::new();

            while let Some(m) = self.building.pop() {
                if m == module.as_str() {
                    break;
                }

                cycle.push(m);
            }

            // create a nice error message
            let mut message = module.as_str().to_string();
            let iter = cycle.iter().rev();
            for name in iter {
                write!(message, " -> {}", name).unwrap();
            }

            return Err(vec![Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!(
                        "Circular import detected in {}.sero:\n{} -> {}\n",
                        module.as_str(),
                        message,
                        module.as_str()
                    ),
                },
                module.as_span(),
            )]);
        }
        self.building.push(module.as_str());

        match LIBRARIES.get_file(module.as_str().to_string() + ".sero") {
            Some(file) => {
                let content = file.contents_utf8().unwrap();

                let pairs = match PestParser::parse(Rule::module, content) {
                    Ok(pairs) => pairs,
                    Err(e) => {
                        return Err(vec![e]);
                    }
                };

                let ast = parse_module_ast(module.as_str().to_string(), pairs)?;

                for import in ast.imports.clone() {
                    self._resolve(import)?;
                }

                self.asts.insert(module.as_str(), ast);
                self.building.pop();
            }
            None => {
                return Err(vec![Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("Could not find module {}", module.as_str()),
                    },
                    module.as_span(),
                )]);
            }
        };

        Ok(())
    }
}

// Lifecycle of a module:
// - Simple parse the module according to the grammar
// - Parse the module's dependencies
// - Replace all function calls with their fully-qualified names
// AST
//     imports: Vec<String>,
//     definitions: HashMap<String, Definition>,
// Dependencies
//     imports: Vec<Rc<Module>>,
// FullyQualifiedNames
//     definitions: HashMap<String, Definition>,
// }

/// Functions are executed at runtime by pushing and popping bytes to the stack.
/// There are two special functions
/// - Constants: known values are pushed to the stack
/// - Brainfuck: raw unsafe Brainfuck code is treated as a function
///
/// Compositions are executed at compile time. They take typed functions as arguments and produce new functions.
/// Qoutations are the most common type of composition where a list of functions is joined through concatenatation.
/// - _Technically_ qoutations is function "composition" but to avoid overloading the term I say "concatenatation" or "qoutation"
/// Functions can be defined to become compositions when they are given constants as arguments in certian positions.
/// - This behavior is defined in the source code and requires use of additional syntax.
/// -

#[derive(Debug)]
pub(crate) struct ModuleAst<'a> {
    pub(crate) name: String,
    pub(crate) imports: Vec<Pair<'a, Rule>>,
    pub(crate) definitions: HashMap<String, Vec<Definition<'a>>>,
}

fn parse_module_ast(name: String, pairs: Pairs<Rule>) -> Result<ModuleAst, Vec<Error<Rule>>> {
    // add input str to illicit layer
    let mut imports = Vec::new();
    let mut definitions: HashMap<String, Vec<Definition>> = HashMap::new();

    let mut errors = Vec::new();

    for pair in pairs.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::imports => {
                for pair in pair.into_inner() {
                    imports.push(pair);
                }
            }
            Rule::definition_sequence => {
                for pair in pair.into_inner() {
                    let def = parse_definition_ast(pair);

                    match def {
                        Ok(def) => {
                            if let Some(defs) = definitions.get_mut(&def.name) {
                                defs.push(def);
                            } else {
                                definitions.insert(def.name.clone(), vec![def]);
                            }
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    Ok(ModuleAst {
        name,
        imports,
        definitions,
    })
}

fn parse_definition_ast(pair: Pair<Rule>) -> Result<Definition, Error<Rule>> {
    assert_eq!(pair.as_rule(), Rule::definition);
    let mut pairs = pair.clone().into_inner();

    let name_pair = pairs.next().unwrap();
    let name = name_pair.as_str().to_string();
    // names [a-zA-Z] are reserved
    if name.len() == 1 {
        let c = name.chars().next().unwrap();
        if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
            return Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: String::from(
                        "Single character names matching 'a'..'z' and 'A'..'Z' are reserved",
                    ),
                },
                name_pair.as_span(),
            ));
        }
    }

    let mut stack = vec![];
    let mut stack_pair = None;
    let mut body = None;
    let mut typ = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::definition_type => {
                // "==?" "==!" "=="
                match pair.as_str() {
                    "==?" => typ = Some(DefinitionType::Composition),
                    "==!" => typ = Some(DefinitionType::ConstantComposition),
                    "==" => {
                        if stack.is_empty() {
                            typ = Some(DefinitionType::Function)
                        } else {
                            typ = Some(DefinitionType::InlineComposition)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Rule::stack => {
                stack_pair = Some(pair.clone());

                // Map position letter to position number
                let mut positions = HashMap::new();
                let mut position_count = 0;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::stack_constant => {
                            let letter = pair.as_str().chars().next().unwrap();

                            if let Some(pos) = positions.get(&letter) {
                                stack.push(StackArg::Position(letter, *pos))
                            } else {
                                positions.insert(letter, position_count);
                                stack.push(StackArg::Position(letter, position_count));
                            }

                            position_count += 1;
                        }
                        Rule::stack_qoutation => {
                            let letter = pair.as_str().chars().next().unwrap();

                            if let Some(pos) = positions.get(&letter) {
                                stack.push(StackArg::Position(letter, *pos))
                            } else {
                                positions.insert(letter, position_count);
                                stack.push(StackArg::Position(letter, position_count));
                            }
                        }
                        Rule::stack_byte => {
                            stack.push(StackArg::Byte(pair.as_str().parse::<u8>().unwrap()))
                        }
                        Rule::stack_ignored_constant => stack.push(StackArg::IgnoredConstant),
                        Rule::stack_ignored_qoutation => stack.push(StackArg::IgnoredQoutation),
                        _ => unreachable!(),
                    }
                }
            }
            Rule::term => {
                body = Some(parse_term_ast(pair)?);
            }
            _ => unreachable!(),
        }
    }

    let stack = match stack_pair {
        Some(pair) => Some(StackArgs { args: stack, pair }),
        None => None,
    };

    Ok(Definition {
        pair,
        name,
        stack,
        body: body.unwrap(),
        typ: typ.unwrap(),
    })
}

fn parse_term_ast(pair: Pair<Rule>) -> Result<Vec<Expression>, Error<Rule>> {
    assert_eq!(pair.as_rule(), Rule::term);

    pair.into_inner()
        .map(|pair| Expression::try_from(pair))
        .collect()
}

fn constant_from_char(pair: Pair<Rule>) -> Expression {
    let b = match pair.as_rule() {
        Rule::char => pair.as_str().bytes().next().unwrap(),
        Rule::raw_char => pair.as_str().bytes().next().unwrap(),
        Rule::escaped => {
            match pair.as_str() {
                // "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t" | "0"
                "\\\\" => b'\\',
                "\\\"" => b'\"',
                "\\b" => 8,
                "\\f" => 12,
                "\\n" => b'\n',
                "\\r" => b'\r',
                "\\t" => b'\t',
                "\\0" => 0,
                _ => unreachable!(),
            }
        }
        Rule::escaped_hex => u8::from_str_radix(&pair.as_str()[2..], 16).unwrap(),
        _ => unreachable!(),
    };

    Expression::Constant(b, pair.as_span())
}
