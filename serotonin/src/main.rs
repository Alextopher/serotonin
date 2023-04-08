use std::rc::Rc;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use colored::Colorize;
use lasso::RodeoReader;
use serotonin_parser::lex;
use serotonin_parser::{InternedToken, Token, TokenData};

fn main() {
    // Read std library
    let std = std::fs::read_to_string(
        "/home/mahonec/p/github.com/Alextopher/serotonin/libraries/std.sero",
    )
    .unwrap();
    lex_debug(&std)
}

fn lex_debug(content: &str) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("std", content);

    let mut rodeo = lasso::Rodeo::default();

    let (tokens, errors) = lex(content, file_id, &mut rodeo);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for error in errors {
        let diagnostic: Diagnostic<usize> = error.into();

        term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }

    let reader = rodeo.into_reader();

    println!("{}", pretty(&tokens, &reader));
    //println!("\n\n\n");
    //println!("{}", debug(&tokens, &reader));
}

// print a Vec<InternedToken> in a nice way to check if the lexer is working
fn pretty(tokens: &[Rc<InternedToken>], reader: &RodeoReader) -> String {
    let mut out = String::new();

    for token in tokens {
        match token.data() {
            TokenData::None => {
                // If the token is Error print it in red using the colored crate
                out.push_str(&match token.kind() {
                    Token::Error => reader.resolve(&token.spur()).red().to_string(),
                    Token::Comment => reader.resolve(&token.spur()).dimmed().to_string(),
                    Token::Whitespace
                    | Token::Substitution
                    | Token::Generation
                    | Token::Execution
                    | Token::LParen
                    | Token::RParen
                    | Token::LBracket
                    | Token::RBracket
                    | Token::Semicolon => reader.resolve(&token.spur()).to_string(),
                    Token::UnnamedByte | Token::UnnamedQuotation => {
                        format!("{}", reader.resolve(&token.spur()).to_string().cyan())
                    }
                    _ => format!("{:?}", token.kind())
                        .to_uppercase()
                        .underline()
                        .to_string(),
                });
            }
            TokenData::Integer(num) => {
                // print out the number in blue using the colored crate
                out.push_str(&format!("{}", num.to_string().purple()));
            }
            TokenData::String(s) => {
                // Add back removed symbols
                let s = match token.kind() {
                    Token::String => format!("\"{}\"", reader.resolve(s)).green(),
                    Token::RawString => format!("\"{}\"", reader.resolve(s)).green(),
                    Token::Brainfuck => format!("`{}`", reader.resolve(s)).yellow(),
                    Token::MacroInput => format!("{{{}}}", reader.resolve(s)).yellow(),
                    Token::NamedByte | Token::NamedQuotation => reader.resolve(s).cyan().bold(),
                    Token::Identifier => reader.resolve(s).cyan(),
                    _ => unreachable!(),
                }
                .to_string();

                out += &s;
            }
        }
    }

    out
}

fn debug(tokens: &[Rc<InternedToken>], reader: &RodeoReader) -> String {
    let mut out = String::new();

    for token in tokens {
        out.push_str(&format!(
            "|{:?}:{}|\n",
            token.kind(),
            reader.resolve(&token.spur())
        ))
    }

    out
}
