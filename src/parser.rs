use std::collections::HashMap;

use pest::{iterators::Pair, Parser, error::{Error, ErrorVariant}, Span};

use crate::stdlib::LIBRARIES;

#[derive(Parser)]
#[grammar = "joy.pest"]
pub struct JoyParser;

// joy is a functional language and at it's base it is made up of two things
// 1. compositions which take some amount of quotations and preforms an operation
// 2. quotations which are "lists" of compositions or quotations and ard typicall preformed in order
// there is some other stuff needed so that we can define new compositions
// and to handle imports and exports
#[derive(Debug)]
pub struct Module<'a> {
    name: String,
    scopes: Vec<String>,
    private: HashMap<String, Definition<'a>>,
    public: HashMap<String, Definition<'a>>,
}

#[derive(Debug)]
pub struct Definition<'a> {
    name: String,
    body: AstNode,
    span: Span<'a>,
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
    modules: HashMap<String, Module<'a>>,
    definitions: HashMap<String, Definition<'a>>,
    // compositions is a list of compositions and the number of qoutations they require
    compositions: HashMap<&'a str, usize>,
    input: &'a str,
}

impl <'a> BFJoyParser<'a> {
    pub fn new() -> BFJoyParser<'a> {
        // Build composition map
        let mut compositions = HashMap::new();

        compositions.insert("while", 2);
        compositions.insert("ifte", 3);

        BFJoyParser {
            modules: HashMap::new(),
            definitions: HashMap::new(),
            compositions,
            input: "",
        }
    }

    pub fn module(&'a mut self, contents: &'a str, name: String) -> Result<Module, Error<Rule>> {
        // copy the input so we can reference it later
        self.input = contents;

        // parse the module with pest
        let result = JoyParser::parse(Rule::module, contents);
        if let Err(e) = result {
            return Err(e);
        }

        let mut count_definitions = 0;
        let mut definitions: (HashMap<String, Definition>, HashMap<String, Definition>) =
            (HashMap::new(), HashMap::new());
        let mut scopes = Vec::new();

        // iterate over the pairs
        for pair in result.unwrap().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::includes => {
                    // parse the includes
                    for name in pair.into_inner() {
                        // check if the module has been loaded
                        if self.modules.contains_key(name.as_str()) {
                            continue;
                        } else {
                            let file = LIBRARIES.get_file(name.as_str().to_string() + ".joy");
                            match file {
                                Some(file) => {
                                    // read the file
                                    let contents = file.contents_utf8().unwrap();

                                    // parse the module
                                    let module =
                                        self.module(contents, name.as_str().to_string()).unwrap();

                                    // add the module to the modules
                                    self.modules.insert(name.as_str().to_string(), module);
                                }
                                None => {
                                    return Err(Error::new_from_span(
                                        ErrorVariant::CustomError { message: format!("Module not found") },
                                        name.as_span(),
                                    ));
                                }
                            }
                        }

                        // add the module to the scopes
                        scopes.push(name.as_str().to_string());
                    }
                }
                Rule::definition_sequence => match self.definition_sequence(pair) {
                    Ok(definition) => match count_definitions {
                        0 => {
                            definitions.0 = definition;
                            count_definitions += 1;
                        }
                        1 => {
                            definitions.1 = definition;
                            count_definitions += 1;
                        }
                        _ => unreachable!(),
                    },
                    Err(err) => return Err(err),
                },
                Rule::EOI => (),
                _ => panic!("Unexpected rule {:?}", pair),
            }
        }

        // create the scope
        Ok(match count_definitions {
            1 => Module {
                name,
                scopes,
                private: definitions.1,
                public: definitions.0,
            },
            2 => Module {
                name,
                scopes,
                private: definitions.0,
                public: definitions.1,
            },
            _ => unreachable!(),
        })
    }

    fn definition_sequence(
        &'a mut self,
        pair: Pair<Rule>,
    ) -> Result<HashMap<String, Definition>, Error<Rule>> {
        let mut definitions = HashMap::new();

        for pair in pair.into_inner() {
            // extract the span
            let span = pair.as_span();

            match self.definition(pair) {
                Ok(definition) => {
                    // check if the definition has already been defined
                    if definitions.contains_key(&definition.name) {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: format!("Definition already defined"),
                            },
                            span,
                        ));
                    } else {
                        // add the definition to the definitions
                        definitions.insert(definition.name.clone(), definition);
                    }
                }
                Err(err) => return Err(err),
            }
        }

        Ok(definitions)
    }

    fn definition(&'a mut self, pair: Pair<Rule>) -> Result<Definition, Error<Rule>> {
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_string();
        let pair = pairs.next().unwrap();
        
        let mut stack = Vec::new();

        match self.qoutation(&mut stack, pair) {
            Some(err) => Err(err),
            None => Ok(Definition {
                name,
                body: stack.pop().unwrap(),
                span: Span::new(self.input, span.start(), span.end()).unwrap(),
            }),
        }
    }

    fn qoutation(&'a mut self, stack: &mut Vec<AstNode>, pair: Pair<Rule>) -> Option<Error<Rule>> {
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
                Err(err) => return Some(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("{}", err),
                    },
                    pair.as_span(),
                )),
            },
            Rule::string => stack.push(AstNode::String(pair.as_str().to_string())),
            _ => panic!("Unexpected rule {:?} in qoutation", pair.as_str()),
        };
        None
    }
}
