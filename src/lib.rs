use crate::{
    parser::{Dependencies, PestParser, Rule},
    semantic::apply_semantics,
};
use colored::Colorize;
use config::Config;
use include_dir::{include_dir, Dir};
use pest::{
    error::{Error, ErrorVariant},
    Parser, Span,
};
use std::{rc::Rc, sync::atomic::AtomicUsize};

static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");
const MAX_ITERATIONS: u64 = 10_000_000;

extern crate pest;
extern crate pest_derive;

mod bfoptimizer;
pub mod config;
pub(crate) mod definition;
pub(crate) mod gen;
pub(crate) mod parser;
pub(crate) mod semantic;

pub fn compile(name: &str, input: &str, config: Config) -> Result<String, Vec<Error<Rule>>> {
    if config.timings {
        compile_with_timings(name, input, config)
    } else {
        compile_without_timings(name, input, config)
    }
}

fn compile_with_timings(
    name: &str,
    input: &str,
    config: Config,
) -> Result<String, Vec<Error<Rule>>> {
    // Time sections
    let mut start = std::time::Instant::now();
    let full_start = start;

    let pairs = match PestParser::parse(Rule::module, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(vec![e]),
    };

    let id = Rc::new(AtomicUsize::new(0));

    if config.timings {
        println!("Pest parsing took: {}us", start.elapsed().as_micros());
        start = std::time::Instant::now();
    }

    let this = parser::parse_module_ast(name.to_owned(), pairs, id.clone())?;

    println!("Built top AST in: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let mut asts = Dependencies::resolve(&this, id)?;
    asts.insert(name, this);

    println!(
        "Parsing imports and build ASTs took: {}us",
        start.elapsed().as_micros()
    );
    start = std::time::Instant::now();

    let new_ast = apply_semantics(&mut asts)?;

    println!(
        "Checking all semantics took: {}us",
        start.elapsed().as_micros()
    );
    start = std::time::Instant::now();

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
                    (&mains[1].span).into(),
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

    println!("Found main in: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let code = match gen::gen_main(&new_ast, main) {
        Ok(c) => Ok(c),
        Err(e) => Err(vec![e]),
    }?;

    println!("Gen took: {}us", start.elapsed().as_micros());
    println!("Compiling took: {}us", full_start.elapsed().as_micros());

    if config.optimize {
        Ok(bfoptimizer::optimize_bf(&code))
    } else {
        Ok(code)
    }
}

fn compile_without_timings(
    name: &str,
    input: &str,
    config: Config,
) -> Result<String, Vec<Error<Rule>>> {
    let pairs = match PestParser::parse(Rule::module, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(vec![e]),
    };

    let id = Rc::new(AtomicUsize::new(0));
    let this = parser::parse_module_ast(name.to_owned(), pairs, id.clone())?;

    let mut asts = Dependencies::resolve(&this, id)?;
    asts.insert(name, this);

    let new_ast = apply_semantics(&mut asts)?;

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
                    (&mains[1].span).into(),
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

    let code = match gen::gen_main(&new_ast, main) {
        Ok(c) => Ok(c),
        Err(e) => Err(vec![e]),
    }?;

    if config.optimize {
        Ok(bfoptimizer::optimize_bf(&code))
    } else {
        Ok(code)
    }
}

#[cfg(test)]
mod test_propagation;
#[cfg(test)]
mod test_std;
