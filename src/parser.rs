use core::panic;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser, Span,
};

use crate::stdlib::LIBRARIES;

#[derive(Parser)]
#[grammar = "joy.pest"]
pub struct JoyParser;

#[derive(Debug)]
struct MySpan {
    start: usize,
    end: usize,
}

impl From<Span<'_>> for MySpan {
    fn from(span: Span) -> Self {
        MySpan {
            start: span.start(),
            end: span.end(),
        }
    }
}

// joy is a functional language and at it's base it is made up of two things
// 1. compositions which take some amount of quotations and preforms an operation
// 2. quotations which are "lists" of compositions or quotations and ard typicall preformed in order
// there is some other stuff needed so that we can define new compositions
// and to handle imports and exports
#[derive(Debug)]
pub struct Module {
    name: String,
    scopes: Vec<Rc<Module>>,
    definitions: HashMap<String, Rc<Definition>>,
}

impl Module {}

#[derive(Debug)]
pub struct Definition {
    name: String,
    body: AstNode,
    dependencies: HashSet<String>,
    span: MySpan,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AstNode {
    Qoutation(Vec<AstNode>),
    Composition(String, Vec<AstNode>),
    // Atomic is a special composition that requires zero qoutations
    Atomic(String),
    // Byte is a special composition that adds a single byte to the stack
    Byte(u8),
    // String is a special composition that adds a string to the stack
    String(String),
}

pub struct BFJoyParser<'a> {
    modules: HashMap<String, Rc<Module>>,
    building: Vec<String>,
    compositions: HashMap<&'a str, usize>,
    inputs: Vec<&'a str>,
}

impl<'a> BFJoyParser<'a> {
    pub fn new() -> BFJoyParser<'a> {
        // Build composition map
        let mut compositions = HashMap::new();

        compositions.insert("while", 2);
        compositions.insert("ifte", 3);

        BFJoyParser {
            modules: HashMap::new(),
            building: Vec::new(),
            compositions,
            inputs: Vec::new(),
        }
    }

    fn make_span(&self, span: &MySpan) -> Span<'a> {
        Span::new(self.inputs.last().unwrap(), span.start, span.end).unwrap()
    }

    pub fn module(&mut self, contents: &'a str, name: String) -> Result<Rc<Module>, Error<Rule>> {
        // copy the input so we can reference it later
        self.inputs.push(contents);

        // parse the module with pest
        let result = JoyParser::parse(Rule::module, contents);
        if let Err(e) = result {
            return Err(e);
        }

        let mut definitions = HashMap::new();
        let mut scopes = Vec::new();

        // iterate over the pairs
        for pair in result.unwrap().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::imports => {
                    // parse the imports
                    for name in pair.into_inner() {
                        // check if we are already building this module
                        if self.building.contains(&name.as_str().to_string()) {
                            // build the cycle so it can be reported
                            let mut cycle = Vec::new();
                            while self.building.last().unwrap() != name.as_str() {
                                cycle.push(self.building.pop().unwrap());
                            }

                            // create a nice error message
                            let mut message = format!("{}", name.as_str());
                            let iter = cycle.iter().rev();
                            for name in iter {
                                message.push_str(&format!(" -> {}", name));
                            }

                            return Err(Error::new_from_span(
                                ErrorVariant::CustomError {
                                    message: format!(
                                        "Circular import detected:\n{} -> {}\n",
                                        message,
                                        name.as_str()
                                    ),
                                },
                                name.as_span(),
                            ));
                        }

                        // check if the module has been loaded
                        let module = if self.modules.contains_key(name.as_str()) {
                            self.modules.get(name.as_str()).unwrap().to_owned()
                        } else {
                            let file = LIBRARIES.get_file(name.as_str().to_string() + ".joy");
                            match file {
                                Some(file) => {
                                    // read the file
                                    let contents = file.contents_utf8().unwrap();

                                    // track that we are currently building this module
                                    self.building.push(name.as_str().to_string());

                                    // parse the module
                                    let module =
                                        match self.module(contents, name.as_str().to_string()) {
                                            Ok(module) => module,
                                            Err(e) => {
                                                return Err(e);
                                            }
                                        };

                                    // track that we are no longer building this module
                                    self.building.pop();

                                    module
                                }
                                None => {
                                    return Err(Error::new_from_span(
                                        ErrorVariant::CustomError {
                                            message: format!("Could not find module"),
                                        },
                                        name.as_span(),
                                    ));
                                }
                            }
                        };

                        scopes.push(module.clone());
                        self.modules.insert(name.as_str().to_string(), module);
                    }
                }
                Rule::definition_sequence => match self.definition_sequence(pair) {
                    Ok(defs) => definitions = defs,
                    Err(err) => return Err(err),
                },
                Rule::EOI => (),
                _ => panic!("Unexpected rule {:?}", pair),
            }
        }
        self.inputs.pop();

        let module = Rc::new(Module {
            name: name.clone(),
            scopes,
            definitions,
        });

        self.modules
            .insert(name.as_str().to_string(), module.clone());

        Ok(module)
    }

    fn definition_sequence(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<HashMap<String, Rc<Definition>>, Error<Rule>> {
        let mut definitions = HashMap::new();

        for pair in pair.into_inner() {
            // extract the span
            let span = pair.as_span();

            match self.definition(pair) {
                Ok(definition) => {
                    // check if the definition has already been defined
                    match definitions.insert(definition.name.clone(), definition) {
                        Some(def) => {
                            let old_line = self.make_span(&def.span).start_pos().line_col().0;
                            let new_line = span.start_pos().line_col().0;

                            return Err(Error::new_from_span(
                                ErrorVariant::CustomError {
                                    message: format!("def {:?} redefined on line {}. It was originally defined on line {}", def.name, new_line, old_line ),
                                },
                                span
                            ));
                        }
                        None => {}
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(definitions)
    }

    fn definition(&mut self, pair: Pair<Rule>) -> Result<Rc<Definition>, Error<Rule>> {
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_string();
        let pair = pairs.next().unwrap();

        let mut stack = Vec::new();

        match self.qoutation(&mut stack, pair) {
            Some(err) => Err(err),
            None => {
                let body = stack.pop().unwrap();
                let span = span.into();

                // track dependencies (but do not resolve them)
                let mut dependencies = HashSet::new();
                let mut stack = vec![&body];

                while let Some(qoutation) = stack.pop() {
                    match qoutation {
                        AstNode::Qoutation(qoutations) => stack.extend(qoutations),
                        AstNode::Composition(_, qoutations) => stack.extend(qoutations),
                        AstNode::Atomic(name) => {
                            dependencies.insert(name.to_string());
                        }
                        _ => {}
                    }
                }

                Ok(Rc::new(Definition {
                    name,
                    body,
                    dependencies,
                    span,
                }))
            }
        }
    }

    fn qoutation(&mut self, stack: &mut Vec<AstNode>, pair: Pair<Rule>) -> Option<Error<Rule>> {
        match pair.as_rule() {
            Rule::term => {
                let mut sub_stack = Vec::new();

                for term in pair.into_inner() {
                    self.qoutation(&mut sub_stack, term);
                }

                stack.push(AstNode::Qoutation(sub_stack));
            }
            Rule::atomic => {
                let name = pair.as_str();

                // check if the composition is in our list of special compositions
                match self.compositions.get(name) {
                    Some(qoutations) => {
                        // slice qoutations off the stack (in reverse order)
                        let qoutations = stack.split_off(stack.len() - qoutations);
                        stack.push(AstNode::Composition(name.to_string(), qoutations));
                    }
                    None => stack.push(AstNode::Atomic(name.to_string())),
                }
            }
            Rule::integer => match pair.as_str().parse::<u8>() {
                Ok(byte) => stack.push(AstNode::Byte(byte)),
                Err(err) => {
                    return Some(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: format!("{}", err),
                        },
                        pair.as_span(),
                    ))
                }
            },
            Rule::string => stack.push(AstNode::String(pair.as_str().to_string())),
            _ => panic!("Unexpected rule {:?} in qoutation", pair.as_str()),
        };

        None
    }

    // Create a topological ordering of the definitions using DFS
    pub fn create_topological_order(&self, module: Rc<Module>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut working = HashSet::new();
        let mut order = Vec::new();

        match module.definitions.keys().into_iter().try_for_each(|n| {
            self.visit(
                &mut visited,
                &mut working,
                &mut order,
                format!("{}.{}", &module.name, n),
            )
        }) {
            Ok(_) => Some(order),
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }

    fn visit(
        &self,
        visited: &mut HashSet<String>,
        working: &mut HashSet<String>,
        order: &mut Vec<String>,
        name: String,
    ) -> Result<(), String> {
        if visited.contains(&name) {
            return Ok(());
        }

        if working.contains(&name) {
            return Err(format!("Cyclic dependency detected: {}", name));
        }

        working.insert(name.clone());

        // split the name into the module and the definition
        let mut parts = name.split('.');
        let module = self.modules.get(parts.next().unwrap()).unwrap();
        let def = module.definitions.get(parts.next().unwrap()).unwrap();

        for dep in &def.dependencies {
            // find a suitable definition in scope
            let mut found = false;

            // first check if the definition is in the same module
            if let Some(def) = module.definitions.get(dep) {
                if let Err(e) = self.visit(
                    visited,
                    working,
                    order,
                    format!("{}.{}", &module.name, def.name),
                ) {
                    return Err(e);
                }

                found = true;
            } else {
                for scope in module.scopes.iter().rev() {
                    if let Some(def) = scope.definitions.get(dep) {
                        if let Err(e) = self.visit(
                            visited,
                            working,
                            order,
                            format!("{}.{}", &scope.name, def.name),
                        ) {
                            return Err(e);
                        }

                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return Err(format!(
                    "Could not find definition for {} in module {}\nScopes: {:?}",
                    dep,
                    module.name,
                    module.scopes.iter().map(|s| &s.name).collect::<Vec<_>>()
                ));
            }
        }

        working.remove(&name);
        visited.insert(name.clone());
        order.push(name);

        Ok(())
    }
}
