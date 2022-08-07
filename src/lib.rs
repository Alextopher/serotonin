#![feature(pointer_byte_offsets)]

use std::{rc::Rc, sync::atomic::AtomicUsize};

use crate::{
    parser::{Dependencies, PestParser, Rule},
    semantic::apply_semantics,
};
use colored::Colorize;
use pest::{
    error::{Error, ErrorVariant},
    Parser, Span,
};

use include_dir::{include_dir, Dir};
static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");

extern crate clap;
extern crate pest;
extern crate pest_derive;

pub(crate) mod bfoptimizer;
pub mod config;
pub(crate) mod definition;
pub(crate) mod gen;
pub mod parser;
pub(crate) mod semantic;
// pub mod typ;

pub fn compile(name: &str, input: &str) -> Result<String, Vec<Error<Rule>>> {
    // Time sections
    let mut start = std::time::Instant::now();
    let full_start = start;

    let pairs = match PestParser::parse(Rule::module, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(vec![e]),
    };

    let id = Rc::new(AtomicUsize::new(0));

    println!("Pest parsing took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

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

    let c = match gen::gen_main(&new_ast, main) {
        Ok(c) => Ok(c),
        Err(e) => Err(vec![e]),
    }?;

    println!("Compiling took: {}us", start.elapsed().as_micros());
    start = std::time::Instant::now();

    let code = bfoptimizer::optimize_bf(c);
    println!("Optimizing took: {}us", start.elapsed().as_micros());

    println!(
        "Full compilation took: {}ms",
        full_start.elapsed().as_millis()
    );

    Ok(code)
}

#[cfg(test)]
mod tests;
