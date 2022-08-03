use colored::Color;
use colored::Colorize;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::Span;
use std::collections::HashMap;

use crate::parser::StackArg;
use crate::parser::{Definition, Expression, ModuleAst, Rule, StackArgs};

// Preform semantic analysis on a Module.
// - The same name and stack argument can not be used twice.
// - Verify that all functions are defined within scope
// - Where applicable replace function calls with fully qualified names
pub(crate) fn apply_semantics<'a>(
    asts: &'a mut HashMap<&str, ModuleAst>,
) -> Result<HashMap<&'a str, ModuleAst<'a>>, Vec<Error<Rule>>> {
    let mut errors: Vec<Error<Rule>> = Vec::new();

    // The same name and stack arguments can not be duplicated.
    for (module, ast) in asts.iter_mut() {
        // Definitions that fail this test are removed from the AST.
        let mut remove = Vec::new();

        for (name, defs) in ast.definitions.iter() {
            // Create a HashMap from stack effect to definition
            // When inserting if we detect a duplicate stack effect we report an error and continue
            let mut stacks: HashMap<Option<StackArgs>, &Definition> = HashMap::new();

            let mut stack_errors: HashMap<Option<StackArgs>, Vec<Error<Rule>>> = HashMap::new();
            let mut originals: HashMap<Option<StackArgs>, Error<Rule>> = HashMap::new();

            for def in defs.iter().rev() {
                if let Some(original) = stacks.get(&def.stack) {
                    // If the errors map does not contain any entries yet add the original def
                    match stack_errors.get_mut(&def.stack) {
                        Some(v) => v.push(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!(
                                    "Duplicate definition for {} {} in {}.",
                                    name.color(Color::Red),
                                    def.stack_as_str().color(Color::Blue),
                                    module.color(Color::Green)
                                )
                                .bold()
                                .to_string(),
                            },
                            def.pair.as_span(),
                        )),
                        None => {
                            stack_errors.insert(
                                def.stack.clone(),
                                vec![Error::new_from_span(
                                    ErrorVariant::CustomError {
                                        message: format!(
                                            "Duplicate definition for {} {} in {}.",
                                            name.color(Color::Red),
                                            def.stack_as_str().color(Color::Blue),
                                            module.color(Color::Green)
                                        )
                                        .bold()
                                        .to_string(),
                                    },
                                    def.pair.as_span(),
                                )],
                            );
                            originals.insert(
                                def.stack.clone(),
                                Error::new_from_span(
                                    ErrorVariant::CustomError {
                                        message: format!(
                                            "{} {} originally defined here.",
                                            name.color(Color::Red),
                                            original.stack_as_str().color(Color::Blue)
                                        )
                                        .bold()
                                        .to_string(),
                                    },
                                    original.pair.as_span(),
                                ),
                            );
                        }
                    }
                } else {
                    stacks.insert(def.stack.clone(), def);
                }
            }

            if !stack_errors.is_empty() {
                // Merge errors into the main error vector
                for (stack, stack_errors) in stack_errors {
                    errors.extend(stack_errors.into_iter().rev());
                    errors.push(originals.get(&stack).unwrap().to_owned());
                }

                // Mark the definition for removal
                remove.push(name.clone());
            }
        }

        // Remove the definitions that failed this test
        remove.iter().for_each(|name| {
            ast.definitions.remove(name);
        });
    }

    let mut new_asts: HashMap<&'a str, ModuleAst<'a>> = HashMap::new();

    // Verify that every function used in the module is defined somewhere in scope and replace function calls with fully qualified names.
    for (_, ast) in asts.iter() {
        let mut scopes: Vec<&ModuleAst> = ast
            .imports
            .iter()
            .map(|import| asts.get(import.as_str()).unwrap())
            .collect();
        // Add the current module to the scope list
        scopes.push(ast);

        let mut new_defs = HashMap::new();

        for (name, defs) in ast.definitions.iter() {
            let mut sub_defs = Vec::new();

            for def in defs.iter() {
                // Wrap the body into a quotation
                let mut body = Vec::new();

                for e in def.body.iter() {
                    match qualify_expression(&scopes, &def.stack, e) {
                        Ok(e) => body.push(e),
                        Err(e) => errors.extend(e),
                    }
                }

                sub_defs.push(Definition {
                    typ: def.typ,
                    name: def.name.clone(),
                    stack: def.stack.clone(),
                    body,
                    pair: def.pair.clone(),
                })
            }

            new_defs.insert(name.to_owned(), sub_defs);
        }

        new_asts.insert(
            ast.name.as_str(),
            ModuleAst {
                name: ast.name.clone(),
                imports: ast.imports.clone(),
                definitions: new_defs,
            },
        );
    }

    if errors.is_empty() {
        Ok(new_asts)
    } else {
        Err(errors)
    }
}

// Replaces function calls with fully qualified names
// - Prefer fully qualified functions that are first in scope
// - Cache the results of finding functions to improve performance
fn qualify_expression<'a>(
    scopes: &Vec<&ModuleAst>,
    stack: &Option<StackArgs<'a>>,
    e: &Expression<'a>,
) -> Result<Expression<'a>, Vec<Error<Rule>>> {
    match e {
        Expression::Constant(_, _) | Expression::Brainfuck(_, _) => Ok(e.clone()),
        Expression::Quotation(q, p) => {
            let mut errors = Vec::new();
            let mut quotation = Vec::new();

            for e in q.into_iter() {
                match qualify_expression(scopes, stack, e) {
                    Ok(e) => quotation.push(e),
                    Err(e) => errors.extend(e),
                }
            }

            if errors.is_empty() {
                Ok(Expression::Quotation(quotation, p.clone()))
            } else {
                Err(errors)
            }
        }
        Expression::Function { module, name, span } => {
            // Check if module is "" and name is in stack
            if module.is_empty() && stack.is_some() {
                let stack = stack.as_ref().unwrap();

                for arg in &stack.args {
                    if let StackArg::Position(c, _) = arg {
                        if c.to_string() == *name {
                            return Ok(e.clone());
                        }
                    }
                }
            }

            match qualify_string(scopes, &module, &name, span) {
                Ok((module, name)) => Ok(Expression::Function {
                    module: module.to_string(),
                    name: name.to_string(),
                    span: span.clone(),
                }),
                Err(e) => Err(vec![e]),
            }
        }
    }
}

fn qualify_string<'a>(
    scopes: &Vec<&'a ModuleAst>,
    module_name: &'a str,
    name: &'a str,
    span: &'a Span<'a>,
) -> Result<(&'a str, &'a str), Error<Rule>> {
    let this = scopes.last().unwrap();

    // if s is in the form "module.function"
    // verify the module is in scope
    // verify that the function is defined
    // if the function is defined then we ensure it is accessible
    if module_name != "" {
        if let Some(module) = scopes.iter().find(|m| m.name == module_name) {
            if module.definitions.contains_key(name) {
                // If the function starts with '_' then it must be in "this" module
                if name.starts_with('_') && module_name != this.name {
                    Err(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!(
                                "Function {} is private with module {}",
                                name.color(Color::Red),
                                module_name.color(Color::Green)
                            )
                            .bold()
                            .to_string(),
                        },
                        span.clone(),
                    ))
                } else {
                    Ok((module_name, name))
                }
            } else {
                Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!(
                            "Function {} is not defined in module {}",
                            name.color(Color::Red),
                            module_name.color(Color::Green)
                        )
                        .bold()
                        .to_string(),
                    },
                    span.clone(),
                ))
            }
        } else {
            Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: format!("Module {} was not found", module_name.color(Color::Green))
                        .bold()
                        .to_string(),
                },
                span.clone(),
            ))
        }
    } else {
        // Check each scope in reverse order to find the first definition that matches the name
        for scope in scopes.iter().rev() {
            if let Some(_) = scope.definitions.get(name) {
                return Ok((&scope.name, name));
            }
        }

        // If we get here then the function is not defined
        Err(Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("Function {} is not defined", name.color(Color::Red))
                    .bold()
                    .to_string(),
            },
            span.clone(),
        ))
    }
}
