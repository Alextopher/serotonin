use crate::{
    definition::{Definition, DefinitionType, Expression, StackArg, StackArgs},
    parser::{ModuleAst, Rule},
    MAX_ITERATIONS,
};
use bfi::Error::*;
use colored::Colorize;
use either::Either;
use pest::error::{Error, ErrorVariant};
use std::collections::{HashMap, HashSet};

pub(crate) fn gen_main<'a>(
    modules: &HashMap<String, ModuleAst<'a>>,
    def: &Definition,
) -> Result<String, Error<Rule>> {
    // main function must be a function and have no arguments
    if def.stack.is_some() {
        return Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "main function must have no pattern matches"
                    .bold()
                    .to_string(),
            },
            pest::Span::new(&def.span.data, def.span.start, def.span.end).unwrap(),
        ));
    }

    let mut builds = HashSet::new();
    let constraints = HashMap::new();
    compile(modules, def, constraints, &mut builds)
}

pub(crate) fn compile<'a>(
    modules: &HashMap<String, ModuleAst<'a>>,
    def: &Definition,
    constraints: HashMap<char, Expression>,
    builds: &mut HashSet<usize>,
) -> Result<String, Error<Rule>> {
    // Add function to the build list
    builds.insert(def.unique_id);

    let result = compile_body(modules, &def.body, &constraints, builds)?;

    builds.remove(&def.unique_id);
    Ok(result)
}

// Alerts the compiler to preform some action during compilation
#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    // Finished compiling a function and we can remove it from the build list
    Finish(usize),
}

fn compile_body<'a>(
    modules: &HashMap<String, ModuleAst<'a>>,
    body: &[Expression],
    constraints: &HashMap<char, Expression>,
    builds: &mut HashSet<usize>,
) -> Result<String, Error<Rule>> {
    let body = apply_constraints(constraints, body);

    let mut work: Vec<Either<Expression, Message>> =
        body.into_iter().rev().map(Either::Left).collect();
    let mut stack: Vec<Expression> = Vec::new();

    // add expression from the body onto the stack
    while let Some(expr) = work.pop() {
        match expr {
            Either::Left(expr) => {
                if let Expression::Function(module, name, span) = expr.clone() {
                    // Look up all definitions for the function in modules
                    let definitions = modules
                        .get(&module)
                        .unwrap()
                        .definitions
                        .get(&name)
                        .unwrap();

                    let mut function: Option<&Definition> = None;
                    let mut new_constraints = None;
                    for def in definitions.iter().rev() {
                        // skip the definition if it is already being built
                        if builds.contains(&def.unique_id) {
                            continue;
                        }

                        // Check if we match the pattern
                        match &def.stack {
                            Some(pattern) => match pattern_match(&stack, pattern) {
                                Some(c) => {
                                    new_constraints = Some(c);
                                    function = Some(def);
                                    break;
                                }
                                None => continue,
                            },
                            None => {
                                function = Some(def);
                                break;
                            }
                        }
                    }

                    let new_constraints = match new_constraints {
                        Some(c) => c,
                        None => HashMap::new(),
                    };

                    // Depending on the function we have more work to do.
                    if let Some(def) = function {
                        match def.typ {
                            DefinitionType::Function => {
                                // Add the function body to the work list
                                builds.insert(def.unique_id);
                                work.push(Either::Right(Message::Finish(def.unique_id)));
                                work.extend(def.body.iter().rev().cloned().map(Either::Left));
                            }
                            DefinitionType::InlineComposition => {
                                // Extend the work list with the body of the function
                                builds.insert(def.unique_id);
                                work.push(Either::Right(Message::Finish(def.unique_id)));
                                work.extend(
                                    apply_constraints(&new_constraints, &def.body)
                                        .into_iter()
                                        .rev()
                                        .map(Either::Left),
                                );

                                // Remove the pattern from the stack
                                stack.truncate(stack.len() - def.stack_size());
                            }
                            DefinitionType::ConstantComposition => {
                                // Compile the definition with replacing constraints with their values
                                let bf = compile(modules, def, new_constraints, builds)?;

                                // Remove the pattern from the stack
                                stack.truncate(stack.len() - def.stack_size());

                                match bfi::execute(&bf, [], MAX_ITERATIONS) {
                                    Ok(output) => {
                                        for c in output {
                                            stack.push(Expression::Constant(c, expr.span().clone()))
                                        }
                                    }
                                    Err(e) => {
                                        let message = match e {
                                            ParseError(e) => {
                                                format!(
                                                    "Error compiling inline composition {:?}",
                                                    e
                                                )
                                            }
                                            RunTimeError(e) => {
                                                format!(
                                                    "Error executing inline composition {:?}",
                                                    e
                                                )
                                            }
                                        };
                                        return Err(Error::new_from_span(
                                            pest::error::ErrorVariant::CustomError {
                                                message: message.bold().to_string(),
                                            },
                                            expr.span().into(),
                                        ));
                                    }
                                }
                            }
                            DefinitionType::Composition => {
                                // Compile the definition with replacing constraints with their values
                                let bf = compile(modules, def, new_constraints, builds)?;

                                // Remove the pattern from the stack
                                stack.truncate(stack.len() - def.stack_size());

                                // Execute the definition with no inputs
                                match bfi::execute(&bf, [], MAX_ITERATIONS) {
                                    Ok(output) => {
                                        // Push the result on the stack as a bf block
                                        let bf =
                                            String::from_iter(output.iter().map(|x| *x as char));
                                        stack.push(Expression::Brainfuck(bf, expr.span().clone()))
                                    }
                                    Err(e) => {
                                        let message = match e {
                                            ParseError(e) => {
                                                format!(
                                                    "Error compiling inline composition {:?}",
                                                    e
                                                )
                                            }
                                            RunTimeError(e) => {
                                                format!(
                                                    "Error executing inline composition {:?}",
                                                    e
                                                )
                                            }
                                        };
                                        return Err(Error::new_from_span(
                                            pest::error::ErrorVariant::CustomError {
                                                message: message.bold().to_string(),
                                            },
                                            expr.span().into(),
                                        ));
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!(
                                    "No unused definition available for {}. Perhaps there is a circular dependency?",
                                    name.red()
                                )
                                .bold()
                                .to_string(),
                            },
                            (&span).into(),
                        ));
                    }
                } else if let Expression::Quotation(q, s) = expr {
                    // Compile the quotation
                    let bf = compile_body(modules, &q, constraints, builds)?;

                    // Push the result on the stack as a bf block
                    stack.push(Expression::Brainfuck(bf, s.clone()))
                } else if let Expression::Macro(input, method, s) = expr {
                    // The only valid macro is "autoperm!"
                    if method != "autoperm!" {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!(
                                    "Invalid macro {}. Only `autoperm!` is supported",
                                    method.red()
                                )
                                .bold()
                                .to_string(),
                            },
                            (&s).into(),
                        ));
                    }

                    // Run autoperm! algorithm on the input
                    match autoperm::autoperm_bf(&input) {
                        Ok(bf) => stack.push(Expression::Brainfuck(bf, s.clone())),
                        Err(e) => {
                            return Err(Error::new_from_span(
                                ErrorVariant::CustomError {
                                    message: format!("Error executing autoperm! macro: {}", e)
                                        .bold()
                                        .to_string(),
                                },
                                (&s).into(),
                            ));
                        }
                    }
                } else {
                    stack.push(expr.clone());
                }
            }
            Either::Right(msg) => match msg {
                Message::Finish(id) => {
                    builds.remove(&id);
                }
            },
        }
    }

    let result = stack
        .iter()
        .map(|e| e.compiled())
        .collect::<Vec<_>>()
        .join("");

    Ok(result)
}

fn pattern_match(stack: &[Expression], pattern: &StackArgs) -> Option<HashMap<char, Expression>> {
    // Pattern matching match letter to the constants
    let mut constraints: HashMap<char, &Expression> = HashMap::new();
    // Iterate over the stack arguments in reverse order
    let mut expressions = stack.iter().rev();

    let mut matches = true;
    for arg in pattern.args.iter() {
        if let Some(expr) = expressions.next() {
            match (expr, arg) {
                (Expression::Constant(a, _), StackArg::Position(c, _)) => {
                    // Check if c is already constrained
                    if let Some(constraint) = constraints.get(c) {
                        // Verify that the constraint is the same as the constant
                        match constraint {
                            Expression::Constant(constraint, _) => {
                                if constraint != a {
                                    matches = false;
                                    break;
                                }
                            }
                            Expression::Quotation(_, _) => {
                                matches = false;
                                break;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        constraints.insert(*c, expr);
                    }
                }
                (Expression::Constant(a, _), StackArg::Byte(b)) => {
                    if a != b {
                        matches = false;
                    }
                }
                (Expression::Constant(_, _), StackArg::IgnoredConstant) => {
                    // ignore
                }
                (Expression::Brainfuck(s, _), StackArg::Qoutation(c, _)) => {
                    // Check if c is already constrained
                    if let Some(constraint) = constraints.get(c) {
                        // Verify that the constraint is the same as the constant
                        match constraint {
                            Expression::Constant(_, _) => {
                                matches = false;
                                break;
                            }
                            Expression::Brainfuck(constraint, _) => {
                                if constraint != s {
                                    matches = false;
                                    break;
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        constraints.insert(*c, expr);
                    }
                }
                (Expression::Brainfuck(_, _), StackArg::IgnoredQoutation) => {
                    // ignore
                }
                _ => {
                    matches = false;
                    break;
                }
            }
        } else {
            matches = false;
            break;
        }
    }

    // Replace the BF constraints with string quotations
    let mut f = HashMap::new();
    for (c, e) in &constraints {
        match e {
            Expression::Constant(_, _) => {
                f.insert(*c, (*e).clone());
            }
            Expression::Brainfuck(bf, span) => {
                let mut constants = vec![Expression::Constant(0, span.clone())];
                constants.extend(
                    bf.chars()
                        .into_iter()
                        .map(|c| Expression::Constant(c as u8, span.clone())),
                );

                f.insert(*c, Expression::Quotation(constants, span.clone()));
            }
            _ => unreachable!(),
        }
    }

    if matches {
        Some(f)
    } else {
        None
    }
}

fn apply_constraints(
    constraints: &HashMap<char, Expression>,
    expressions: &[Expression],
) -> Vec<Expression> {
    let mut result = vec![];

    for mut expr in expressions {
        match expr {
            Expression::Function(module, name, _) => {
                // If the expression is a stack constraint then we replace it with the correct expression
                if module.is_empty() && name.len() == 1 {
                    if let Some(constraint) = constraints.get(&name.chars().next().unwrap()) {
                        expr = constraint;
                    }
                }

                result.push(expr.clone());
            }
            Expression::Quotation(inner, _) => {
                result.push(Expression::Quotation(
                    apply_constraints(constraints, inner),
                    expr.span().clone(),
                ));
            }
            _ => {
                result.push(expr.clone());
            }
        }
    }

    result
}
