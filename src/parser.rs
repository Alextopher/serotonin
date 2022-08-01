use crate::{pest::Parser, stdlib::LIBRARIES};
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use std::fmt::Write;
use std::{collections::HashMap, iter};

#[derive(Parser)]
#[grammar = "serotonin.pest"]
struct PestParser;

pub fn compile(input: &str) -> Result<String, Error<Rule>> {
    let pairs = PestParser::parse(Rule::module, input)?;
    let main = parse_module_ast(pairs)?;

    let asts = Dependencies::resolve(main)?;
    println!("{:?}", asts.keys());

    Ok(String::new())
}

struct Dependencies<'a> {
    building: Vec<&'a str>,
    asts: HashMap<&'a str, ModuleAst<'a>>,
}

impl<'a> Dependencies<'a> {
    fn resolve(main: ModuleAst<'a>) -> Result<HashMap<&'a str, ModuleAst<'a>>, Error<Rule>> {
        let mut dep = Dependencies {
            building: Vec::new(),
            asts: HashMap::new(),
        };

        for import in main.imports.clone() {
            dep._resolve(import)?;
        }

        Ok(dep.asts)
    }

    fn _resolve(&mut self, module: Pair<'a, Rule>) -> Result<(), Error<Rule>> {
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

            return Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!(
                        "Circular import detected in {}.sero:\n{} -> {}\n",
                        module.as_str(),
                        message,
                        module.as_str()
                    ),
                },
                module.as_span(),
            ));
        }
        self.building.push(module.as_str());

        match LIBRARIES.get_file(module.as_str().to_string() + ".sero") {
            Some(file) => {
                let content = file.contents_utf8().unwrap();

                let pairs = PestParser::parse(Rule::module, content)?;
                let ast = parse_module_ast(pairs)?;

                for import in ast.imports.clone() {
                    self._resolve(import)?;
                }

                self.asts.insert(module.as_str(), ast);
                self.building.pop();
            }
            None => todo!(),
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
pub enum Expression {
    Function(String),
    Constant(u8),
    Brainfuck(String),
    Composition(Vec<Expression>, String),
    Quotation(Vec<Expression>),
}

#[derive(Debug)]
struct ModuleAst<'a> {
    imports: Vec<Pair<'a, Rule>>,
    definitions: HashMap<String, Vec<Definition>>,
}

fn parse_module_ast(pairs: Pairs<Rule>) -> Result<ModuleAst, Error<Rule>> {
    let mut imports = Vec::new();
    let mut definitions: HashMap<String, Vec<Definition>> = HashMap::new();

    for pair in pairs.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::imports => {
                for pair in pair.into_inner() {
                    imports.push(pair);
                }
            }
            Rule::definition_sequence => {
                for pair in pair.into_inner() {
                    let def = parse_definition_ast(pair)?;
                    if let Some(defs) = definitions.get_mut(&def.name) {
                        defs.push(def);
                    } else {
                        definitions.insert(def.name.clone(), vec![def]);
                    }
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    imports.reverse();

    Ok(ModuleAst {
        imports,
        definitions,
    })
}

fn parse_definition_ast(pair: Pair<Rule>) -> Result<Definition, Error<Rule>> {
    assert_eq!(pair.as_rule(), Rule::definition);
    let mut pairs = pair.into_inner();

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
                // Map position letter to position number
                let mut positions = HashMap::new();
                let mut position_count = 0;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::stack_constant => {
                            let letter = pair.as_str().chars().next().unwrap();

                            if let Some(pos) = positions.get(&letter) {
                                stack.push(StackArgs::Position(*pos))
                            } else {
                                positions.insert(letter, position_count);
                                stack.push(StackArgs::Position(position_count));
                            }

                            position_count += 1;
                        }
                        // Rule::stack_qoutation => {
                        //     stack.push(StackArgs::Qoutation(pair.as_str().chars().next().unwrap()))
                        // }
                        Rule::stack_byte => {
                            stack.push(StackArgs::Byte(pair.as_str().parse::<u8>().unwrap()))
                        }
                        Rule::stack_ignored_constant => stack.push(StackArgs::IgnoredConstant),
                        // Rule::stack_ignored_qoutation => stack.push(StackArgs::IgnoredQoutation),
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

    Ok(Definition {
        name,
        stack,
        body: body.unwrap(),
        typ: typ.unwrap(),
    })
}

fn parse_term_ast(pair: Pair<Rule>) -> Result<Vec<Expression>, Error<Rule>> {
    assert_eq!(pair.as_rule(), Rule::term);

    let mut factors: Vec<Expression> = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::atomic => factors.push(Expression::Function(pair.as_str().to_string())),
            Rule::hex_integer => match u8::from_str_radix(&pair.as_str()[2..], 16) {
                Ok(byte) => factors.push(Expression::Constant(byte)),
                Err(err) => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("{}", err),
                        },
                        pair.as_span(),
                    ))
                }
            },
            Rule::integer => match pair.as_str().parse::<u8>() {
                Ok(byte) => factors.push(Expression::Constant(byte)),
                Err(err) => {
                    return Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("{}", err),
                        },
                        pair.as_span(),
                    ))
                }
            },
            Rule::string => factors.extend(
                pair.into_inner()
                    .map(constant_from_char)
                    .chain(iter::once(Expression::Constant(0)))
                    .rev(),
            ),
            Rule::raw_string => factors.extend(pair.into_inner().map(constant_from_char)),
            Rule::brainfuck => factors.push(Expression::Brainfuck(pair.as_str().to_string())),
            Rule::term => {
                // Recurse into the term
                factors.push(Expression::Quotation(parse_term_ast(pair)?));
            }
            _ => unreachable!(),
        };
    }

    return Ok(factors);
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum StackArgs {
    // Lowercase letter
    Position(u8),
    // // Capital letter
    // Qoutation(char),
    // Number
    Byte(u8),
    // -
    IgnoredConstant,
    // // _
    // IgnoredQoutation,
}

#[derive(Debug)]
enum DefinitionType {
    Function,
    InlineComposition,
    ConstantComposition,
    Composition,
}

#[derive(Debug)]
struct Definition {
    name: String,
    stack: Vec<StackArgs>,
    body: Vec<Expression>,
    typ: DefinitionType,
}

fn constant_from_char(rule: Pair<Rule>) -> Expression {
    match rule.as_rule() {
        Rule::char => Expression::Constant(rule.as_str().bytes().next().unwrap()),
        Rule::raw_char => Expression::Constant(rule.as_str().bytes().next().unwrap()),
        Rule::escaped => {
            match rule.as_str() {
                // "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t" | "0")
                "\\\\" => Expression::Constant(b'\\'),
                "\\\"" => Expression::Constant(b'\"'),
                "\\b" => Expression::Constant(8),
                "\\f" => Expression::Constant(12),
                "\\n" => Expression::Constant(b'\n'),
                "\\r" => Expression::Constant(b'\r'),
                "\\t" => Expression::Constant(b'\t'),
                "\\0" => Expression::Constant(0),
                _ => unreachable!(),
            }
        }
        Rule::escaped_hex => {
            // this is exactly 4 characters, the last two are the hex
            Expression::Constant(u8::from_str_radix(&rule.as_str()[2..], 16).unwrap())
        }
        _ => unreachable!(),
    }
}
