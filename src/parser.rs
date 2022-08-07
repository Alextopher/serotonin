use crate::{
    bfoptimizer::optimize_bf,
    definition::{Definition, DefinitionType, Expression, StackArg, StackArgs},
    gen,
    pest::Parser,
    semantic::apply_semantics,
};
use colored::Colorize;
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
    Span,
};
use pest_derive::Parser;
use std::{collections::HashMap, rc::Rc, sync::atomic::AtomicUsize};
use std::{fmt::Write, sync::atomic::Ordering};

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

    let id = Rc::new(AtomicUsize::new(0));

    println!("Pest parsing took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let this = parse_module_ast(name.to_owned(), pairs, id.clone())?;

    println!("Parsing took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let mut asts = Dependencies::resolve(&this, id)?;
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

    // Try to get the main function
    let main = match new_ast.get(name).unwrap().definitions.get("main") {
        Some(mains) => {
            // There can be only 1 main
            if mains.len() > 1 {
                Err(vec![Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: "main function must have no pattern matches"
                            .bold()
                            .to_string(),
                    },
                    mains[1].span,
                )])
            } else {
                Ok(&mains[0])
            }
        }
        None => Err(vec![Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!(
                    "function {} not found in module {}",
                    "main".red(),
                    name.green()
                )
                .bold()
                .to_string(),
            },
            Span::new(input, 0, 0).unwrap(),
        )]),
    }?;

    let c = match gen::gen_main(&new_ast, main) {
        Ok(c) => Ok(c),
        Err(e) => Err(vec![e]),
    }?;

    Ok(optimize_bf(c))
}

use include_dir::{include_dir, Dir};

pub static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");

pub(crate) struct Dependencies<'a> {
    building: Vec<&'a str>,
    asts: HashMap<&'a str, ModuleAst<'a>>,
}

impl<'a> Dependencies<'a> {
    fn resolve(
        main: &ModuleAst<'a>,
        id: Rc<AtomicUsize>,
    ) -> Result<HashMap<&'a str, ModuleAst<'a>>, Vec<Error<Rule>>> {
        let mut dep = Dependencies {
            building: Vec::new(),
            asts: HashMap::new(),
        };

        for import in main.imports.clone() {
            dep._resolve(import, id.clone())?
        }

        Ok(dep.asts)
    }

    fn _resolve(
        &mut self,
        module: Pair<'a, Rule>,
        id: Rc<AtomicUsize>,
    ) -> Result<(), Vec<Error<Rule>>> {
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

                let ast = parse_module_ast(module.as_str().to_string(), pairs, id.clone())?;

                for import in ast.imports.clone() {
                    self._resolve(import, id.clone())?;
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

fn parse_module_ast<'a>(
    name: String,
    pairs: Pairs<'a, Rule>,
    id: Rc<AtomicUsize>,
) -> Result<ModuleAst<'a>, Vec<Error<Rule>>> {
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
                    let def = parse_definition_ast(pair, id.clone());

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

fn parse_definition_ast(pair: Pair<Rule>, id: Rc<AtomicUsize>) -> Result<Definition, Error<Rule>> {
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

                for pair in pair.into_inner().rev() {
                    match pair.as_rule() {
                        Rule::stack_constant => {
                            let letter = pair.as_str().chars().next().unwrap();

                            // Create qoutation constraint
                            if let Some(pos) = positions.get(&letter) {
                                stack.push(StackArg::Position(letter, *pos));
                            } else {
                                positions.insert(letter, position_count);
                                stack.push(StackArg::Position(letter, position_count));
                            }

                            position_count += 1;
                        }
                        Rule::stack_qoutation => {
                            let letter = pair.as_str().chars().next().unwrap();

                            if let Some(pos) = positions.get(&letter) {
                                stack.push(StackArg::Qoutation(letter, *pos))
                            } else {
                                positions.insert(letter, position_count);
                                stack.push(StackArg::Qoutation(letter, position_count));
                            }

                            position_count += 1;
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

    let stack = stack_pair.map(|pair| StackArgs {
        args: stack,
        span: pair.as_span(),
    });

    Ok(Definition {
        name,
        stack,
        body: body.unwrap(),
        typ: typ.unwrap(),
        unique_id: id.fetch_add(1, Ordering::Relaxed),
        span: pair.as_span(),
    })
}

fn parse_term_ast(pair: Pair<Rule>) -> Result<Vec<Expression>, Error<Rule>> {
    assert_eq!(pair.as_rule(), Rule::term);

    pair.into_inner().map(parse_factor_ast).collect()
}

fn parse_factor_ast(pair: Pair<Rule>) -> Result<Expression, Error<Rule>> {
    match pair.as_rule() {
        Rule::atomic => {
            // Either "module.function" or "function"
            if let Some(dot) = pair.as_str().find('.') {
                let (module, name) = pair.as_str().split_at(dot);

                Ok(Expression::Function(
                    String::from(module),
                    String::from(&name[1..]),
                    pair.as_span(),
                ))
            } else {
                Ok(Expression::Function(
                    String::new(),
                    pair.as_str().to_string(),
                    pair.as_span(),
                ))
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
            let mut constants = vec![Expression::Constant(0, pair.as_span().get(0..1).unwrap())];
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

pub(crate) fn constant_from_char(pair: Pair<Rule>) -> Expression {
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
