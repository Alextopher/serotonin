// use core::panic;
// use std::{
//     cell::RefCell,
//     collections::{HashMap, HashSet},
//     fmt::Write,
//     rc::Rc,
// };

// use pest::{
//     error::{Error, ErrorVariant},
//     iterators::Pair,
//     Parser, Span,
// };

// use crate::{
//     config::Config,
//     stdlib::{self, LIBRARIES},
// };

// #[derive(Parser)]
// #[grammar = "joy.pest"]
// pub struct JoyParser;

// #[derive(Debug)]
// struct MySpan {
//     start: usize,
//     end: usize,
// }

// impl From<Span<'_>> for MySpan {
//     fn from(span: Span) -> Self {
//         MySpan {
//             start: span.start(),
//             end: span.end(),
//         }
//     }
// }

// // joy is a functional language and at it's base it is made up of two things
// // 1. compositions which take some amount of quotations and preforms an operation
// // 2. quotations which are "lists" of compositions or quotations and ard typicall preformed in order
// // there is some other stuff needed so that we can define new compositions
// // and to handle imports and exports
// #[derive(Debug)]
// pub struct Module {
//     name: String,
//     scopes: Vec<Rc<Module>>,
//     definitions: HashMap<String, Rc<Definition>>,
// }

// #[derive(Debug)]
// pub struct Definition {
//     name: String,
//     body: RefCell<AstNode>,
//     dependencies: HashSet<String>,
//     span: MySpan,
// }

// #[derive(PartialEq, Eq, Debug, Clone)]
// pub enum AstNode {
//     Qoutation(Vec<AstNode>),
//     Composition(String, Vec<AstNode>),
//     // Atomic is a function that requires zero qoutations
//     Atomic(RefCell<String>),
//     // Byte is a special function that adds a single byte to the stack
//     Byte(u8),
//     // Brainfuck is a special function for that unsafely preforms brainfuck operations
//     Brainfuck(String),
// }

// impl AstNode {
//     // looks through the body of the definition and updates all atomics to their fully qualified names
//     fn qualify_atomics(&mut self, atomic: &str, qualified: &str) {
//         match self {
//             AstNode::Qoutation(compositions) => {
//                 for composition in compositions {
//                     composition.qualify_atomics(atomic, qualified)
//                 }
//             }
//             AstNode::Composition(_, qoutations) => {
//                 for qoutation in qoutations {
//                     qoutation.qualify_atomics(atomic, qualified)
//                 }
//             }
//             AstNode::Atomic(cell) => {
//                 cell.replace_with(|f| {
//                     if f == atomic {
//                         qualified.to_string()
//                     } else {
//                         f.to_string()
//                     }
//                 });
//             }
//             _ => {}
//         }
//     }
// }

// #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
// enum CompileOption {
//     // CodeGolfConstants will use code golfed variants of the 256 constants
//     CodeGolfConstants,
// }

// pub struct BFJoyParser<'a> {
//     modules: HashMap<String, Rc<Module>>,
//     building: Vec<String>,
//     compositions: HashMap<&'a str, (usize, &'a str)>,
//     imports: Vec<&'a str>,
//     generated: HashMap<String, String>,
//     constants: Vec<String>,
//     config: Config,
// }

// impl<'a> BFJoyParser<'a> {
//     fn new(config: Config) -> BFJoyParser<'a> {
//         BFJoyParser {
//             modules: HashMap::new(),
//             building: Vec::new(),
//             compositions: stdlib::load_compositions(),
//             imports: Vec::new(),
//             generated: HashMap::new(),
//             constants: if config.golf {
//                 stdlib::load_code_golfed_constants()
//             } else {
//                 stdlib::load_simple_constants()
//             },
//             config,
//         }
//     }

//     fn make_span(&self, span: &MySpan) -> Span<'a> {
//         Span::new(self.imports.last().unwrap(), span.start, span.end).unwrap()
//     }

//     fn module(&mut self, contents: &'a str, name: String) -> Result<Rc<Module>, Error<Rule>> {
//         // copy the input so we can reference it later
//         self.imports.push(contents);

//         // parse the module with pest
//         let result = JoyParser::parse(Rule::module, contents);
//         if let Err(e) = result {
//             return Err(e);
//         }

//         let mut definitions = HashMap::new();
//         let mut scopes = Vec::new();

//         // iterate over the pairs
//         for pair in result.unwrap().next().unwrap().into_inner() {
//             match pair.as_rule() {
//                 Rule::imports => {
//                     // parse the imports
//                     for name in pair.into_inner() {
//                         // If we are already building this module then we've created a circular dependency and need to report an error
//                         if self.building.contains(&name.as_str().to_string()) {
//                             // build the cycle so it can be reported
//                             let mut cycle = Vec::new();

//                             while self.building.last().unwrap() != name.as_str() {
//                                 cycle.push(self.building.pop().unwrap());
//                             }

//                             // create a nice error message
//                             let mut message = name.to_string();
//                             let iter = cycle.iter().rev();
//                             for name in iter {
//                                 write!(message, " -> {}", name).unwrap();
//                             }

//                             return Err(Error::new_from_span(
//                                 ErrorVariant::CustomError {
//                                     message: format!(
//                                         "Circular import detected:\n{} -> {}\n",
//                                         message,
//                                         name.as_str()
//                                     ),
//                                 },
//                                 name.as_span(),
//                             ));
//                         }

//                         // If we have already parsed this module so we can quickly return a reference to it
//                         // If not then we need to find and parse it.
//                         let module = if let Some(module) = self.modules.get(name.as_str()) {
//                             module.clone()
//                         } else {
//                             let file = LIBRARIES.get_file(name.as_str().to_string() + ".joy");
//                             match file {
//                                 Some(file) => {
//                                     // read the file
//                                     let contents = file.contents_utf8().unwrap();

//                                     // track that we are currently building this module
//                                     self.building.push(name.as_str().to_string());

//                                     // parse the module
//                                     let module =
//                                         match self.module(contents, name.as_str().to_string()) {
//                                             Ok(module) => module,
//                                             Err(e) => {
//                                                 return Err(e);
//                                             }
//                                         };

//                                     // track that we are no longer building this module
//                                     self.building.pop();

//                                     module
//                                 }
//                                 None => {
//                                     // TODO check more locations to find the module
//                                     return Err(Error::new_from_span(
//                                         ErrorVariant::CustomError {
//                                             message: "Could not find module".to_string(),
//                                         },
//                                         name.as_span(),
//                                     ));
//                                 }
//                             }
//                         };

//                         scopes.push(module.clone());
//                         self.modules.insert(name.as_str().to_string(), module);
//                     }
//                 }
//                 // If we are hitting this block then the module has no imports
//                 Rule::definition_sequence => match self.definition_sequence(pair) {
//                     Ok(defs) => definitions = defs,
//                     Err(err) => return Err(err),
//                 },
//                 Rule::EOI => (),
//                 _ => panic!("Unexpected rule {:?}", pair),
//             }
//         }
//         self.imports.pop();

//         let module = Rc::new(Module {
//             name: name.clone(),
//             scopes,
//             definitions,
//         });

//         self.modules
//             .insert(name.as_str().to_string(), module.clone());

//         Ok(module)
//     }

//     fn definition_sequence(
//         &mut self,
//         pair: Pair<Rule>,
//     ) -> Result<HashMap<String, Rc<Definition>>, Error<Rule>> {
//         let mut definitions = HashMap::new();

//         for pair in pair.into_inner() {
//             // extract the span
//             let span = pair.as_span();

//             match self.definition(pair) {
//                 Ok(definition) => {
//                     // check if the definition has already been defined
//                     match definitions.insert(definition.name.clone(), definition) {
//                         Some(def) => {
//                             let old_line = self.make_span(&def.span).start_pos().line_col().0;
//                             let new_line = span.start_pos().line_col().0;

//                             return Err(Error::new_from_span(
//                                 ErrorVariant::CustomError {
//                                     message: format!("def {:?} redefined on line {}. It was originally defined on line {}", def.name, new_line, old_line ),
//                                 },
//                                 span
//                             ));
//                         }
//                         None => {}
//                     }
//                 }
//                 Err(e) => return Err(e),
//             }
//         }

//         Ok(definitions)
//     }

//     fn definition(&mut self, pair: Pair<Rule>) -> Result<Rc<Definition>, Error<Rule>> {
//         let span = pair.as_span();
//         let mut pairs = pair.into_inner();
//         let name = pairs.next().unwrap().as_str().to_string();
//         let pair = pairs.next().unwrap();

//         let mut stack = Vec::new();

//         match self.qoutation(&mut stack, pair) {
//             Some(err) => Err(err),
//             None => {
//                 let body = stack.pop().unwrap();
//                 let span = span.into();

//                 // track dependencies (but do not resolve them)
//                 let mut dependencies = HashSet::new();
//                 let mut stack = vec![&body];

//                 while let Some(qoutation) = stack.pop() {
//                     match qoutation {
//                         AstNode::Qoutation(qoutations) => stack.extend(qoutations),
//                         AstNode::Composition(_, qoutations) => stack.extend(qoutations),
//                         AstNode::Atomic(name) => {
//                             dependencies.insert(name.borrow().to_string());
//                         }
//                         _ => {}
//                     }
//                 }

//                 Ok(Rc::new(Definition {
//                     name,
//                     body: RefCell::new(body),
//                     dependencies,
//                     span,
//                 }))
//             }
//         }
//     }

//     fn qoutation(&mut self, stack: &mut Vec<AstNode>, pair: Pair<Rule>) -> Option<Error<Rule>> {
//         match pair.as_rule() {
//             Rule::term => {
//                 let mut sub_stack = Vec::new();

//                 for term in pair.into_inner() {
//                     self.qoutation(&mut sub_stack, term);
//                 }

//                 stack.push(AstNode::Qoutation(sub_stack));
//             }
//             Rule::atomic => {
//                 let name = pair.as_str();

//                 // Check if the atomic is a builtin composition function such as "if" or "while"
//                 if let Some((qoutations, _)) = self.compositions.get(name) {
//                     let qoutations = stack.split_off(stack.len() - qoutations);
//                     stack.push(AstNode::Composition(name.to_string(), qoutations));
//                 } else {
//                     stack.push(AstNode::Atomic(RefCell::new(name.to_string())))
//                 }
//             }
//             Rule::brainfuck => {
//                 // remove the first and last character
//                 let mut brainfuck = pair.as_str().to_string();
//                 brainfuck.remove(0);
//                 brainfuck.pop();
//                 stack.push(AstNode::Brainfuck(brainfuck));
//             }
//             Rule::integer => match pair.as_str().parse::<u8>() {
//                 Ok(byte) => stack.push(AstNode::Byte(byte)),
//                 Err(err) => {
//                     return Some(Error::new_from_span(
//                         ErrorVariant::CustomError {
//                             message: format!("{}", err),
//                         },
//                         pair.as_span(),
//                     ))
//                 }
//             },
//             Rule::hex_integer => {
//                 // remove the 0x
//                 match u8::from_str_radix(&pair.as_str()[2..], 16) {
//                     Ok(byte) => stack.push(AstNode::Byte(byte)),
//                     Err(err) => {
//                         return Some(Error::new_from_span(
//                             ErrorVariant::CustomError {
//                                 message: format!("{}", err),
//                             },
//                             pair.as_span(),
//                         ))
//                     }
//                 }
//             }
//             Rule::string => pair
//                 .into_inner()
//                 .map(byte_from_char)
//                 .chain(vec![AstNode::Byte(0)])
//                 .rev()
//                 .for_each(|node| stack.push(node)),
//             Rule::char => stack.push(byte_from_char(pair)),
//             _ => panic!("Unexpected rule {:?} in qoutation", pair.as_str()),
//         };

//         None
//     }

//     pub fn generate(&mut self, module: Rc<Module>) -> Result<String, String> {
//         let order = self.create_topological_order(module);

//         if let Err(err) = order {
//             return Err(err);
//         }

//         let codes: Vec<String> = order
//             .unwrap()
//             .iter()
//             .map(|definition| {
//                 let mut parts = definition.split('.');
//                 let module_name = parts.next().unwrap();
//                 let name = parts.next().unwrap();

//                 // get the module
//                 let module = self.modules.get(module_name).unwrap();

//                 // get the definition
//                 let def = module.definitions.get(name).unwrap();

//                 let code = self.generate_qoutation(&def.body.borrow());

//                 self.generated.insert(definition.to_string(), code.clone());

//                 code
//             })
//             .collect();

//         Ok(codes.last().unwrap().to_string())
//     }

//     fn generate_qoutation(&self, node: &AstNode) -> String {
//         match node {
//             AstNode::Qoutation(v) => {
//                 let mut output = vec![];

//                 for qoutation in v {
//                     output.push(self.generate_qoutation(qoutation));
//                 }

//                 output.join("")
//             }
//             AstNode::Composition(composition, qoutations) => {
//                 // look up the composition
//                 let mut composition = self
//                     .compositions
//                     .get(composition.as_str())
//                     .unwrap()
//                     .1
//                     .to_string();

//                 // generate the qoutations
//                 let qoutations = qoutations
//                     .iter()
//                     .map(|qoutation| self.generate_qoutation(qoutation))
//                     .collect::<Vec<_>>();

//                 // build the composition using replacement
//                 for (index, qoutation) in qoutations.iter().enumerate() {
//                     composition = composition.replace(&format!("{{{}}}", index), qoutation);
//                 }

//                 composition
//             }
//             AstNode::Atomic(atomic) => {
//                 let atomic = atomic.borrow();

//                 // return the code of atomic
//                 self.generated.get(&*atomic).unwrap().to_string()
//             }
//             AstNode::Byte(n) => self.constants.get(*n as usize).unwrap().to_string(),
//             AstNode::Brainfuck(code) => code.to_string(),
//         }
//     }

//     // Creates a topological ordering of the fully qualified definitions using DFS. Consider
//     // IMPORT std; main == read pop;
//     // could return "std.read" -> "std.print" -> "std.drop" -> "std.pop" -> "main.main"
//     fn create_topological_order(&self, module: Rc<Module>) -> Result<Vec<String>, String> {
//         let mut visited = HashSet::new();
//         let mut working = HashSet::new();
//         let mut order = Vec::new();

//         match self.visit(
//             &mut visited,
//             &mut working,
//             &mut order,
//             format!("{}.main", module.name),
//         ) {
//             Ok(_) => Ok(order),
//             Err(err) => Err(err),
//         }
//     }

//     fn visit(
//         &self,
//         visited: &mut HashSet<String>,
//         working: &mut HashSet<String>,
//         order: &mut Vec<String>,
//         name: String,
//     ) -> Result<(), String> {
//         if visited.contains(&name) {
//             return Ok(());
//         }

//         if working.contains(&name) {
//             return Err(format!("Cyclic dependency detected for {}", name));
//         }

//         working.insert(name.clone());

//         // split the name into the module and the definition
//         let mut parts = name.split('.');
//         let module = self.modules.get(parts.next().unwrap()).unwrap();
//         let definition = module.definitions.get(parts.next().unwrap()).unwrap();

//         for dep in definition.dependencies.clone() {
//             // find a suitable definition in scope
//             let mut found = false;

//             // check if the definition is in the form "module.name"
//             let parts: Vec<_> = dep.split('.').collect();
//             if parts.len() == 2 {
//                 let scope = self.modules.get(parts[0]).unwrap();

//                 if scope.definitions.get(parts[1]).is_some() {
//                     if let Err(err) = self.visit(visited, working, order, dep.clone()) {
//                         return Err(err);
//                     }

//                     found = true;
//                 }
//             } else if let Some(def) = module.definitions.get(&dep) {
//                 let full_name = format!("{}.{}", &module.name, def.name);

//                 let mut body = definition.body.borrow_mut();
//                 body.qualify_atomics(&dep, &full_name);

//                 // check if the definition is in the same module
//                 if let Err(e) = self.visit(visited, working, order, full_name) {
//                     return Err(e);
//                 }

//                 found = true;
//             } else {
//                 // Check every module in scope to see if it has the definition
//                 for scope in module.scopes.iter().rev() {
//                     if let Some(def) = scope.definitions.get(&dep) {
//                         let full_name = format!("{}.{}", &scope.name, def.name);

//                         let mut body = definition.body.borrow_mut();
//                         body.qualify_atomics(&dep, &full_name);

//                         if let Err(e) = self.visit(visited, working, order, full_name) {
//                             return Err(e);
//                         }

//                         found = true;
//                         break;
//                     }
//                 }
//             }

//             if !found {
//                 return Err(format!(
//                     "Could not find definition for {} in module {}\nScopes: {:?}",
//                     dep,
//                     module.name,
//                     module.scopes.iter().map(|s| &s.name).collect::<Vec<_>>()
//                 ));
//             }
//         }

//         working.remove(&name);
//         visited.insert(name.clone());
//         order.push(name);

//         Ok(())
//     }
// }

// // generates a byte from a character constant
// fn byte_from_char(rule: Pair<Rule>) -> AstNode {
//     match rule.as_rule() {
//         Rule::char => AstNode::Byte(rule.as_str().bytes().next().unwrap()),
//         Rule::escaped => {
//             match rule.as_str() {
//                 // "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
//                 "\\\\" => AstNode::Byte(b'\\'),
//                 "\\\"" => AstNode::Byte(b'\"'),
//                 "\\b" => AstNode::Byte(8),
//                 "\\f" => AstNode::Byte(12),
//                 "\\n" => AstNode::Byte(b'\n'),
//                 "\\r" => AstNode::Byte(b'\r'),
//                 "\\t" => AstNode::Byte(b'\t'),
//                 _ => unreachable!(),
//             }
//         }
//         Rule::escaped_hex => {
//             // this is exactly 4 characters, the last two are the hex
//             AstNode::Byte(u8::from_str_radix(&rule.as_str()[2..], 16).unwrap())
//         }
//         _ => unreachable!(),
//     }
// }
