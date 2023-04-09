use std::{env, path::PathBuf};

use codespan_reporting::{
    diagnostic::Diagnostic,
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use colored::Colorize;
use lasso::RodeoReader;
use serotonin_parser::{lex, parse_module, Span, TokenData, TokenKind, Token};

pub fn lex_debug(file: Option<String>, bench: bool, debug: Option<bool>) {
    let file = file.unwrap_or(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap() + "/../libraries/std.sero")
            .to_str()
            .unwrap()
            .to_string(),
    );

    let debug = debug.unwrap_or(false);

    let content = std::fs::read_to_string(file).unwrap();

    let mut files = SimpleFiles::new();
    let file_id = files.add("std", &content);

    let mut rodeo = lasso::Rodeo::default();

    let (tokens, errors) = lex(&content, file_id, &mut rodeo);

    if bench {
        return;
    }

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for error in errors {
        let diagnostic: Diagnostic<usize> = error.into();

        term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    }

    let reader = rodeo.into_reader();

    if debug {
        println!("{}", debug_print(&tokens, &reader));
    } else {
        println!("{}", pretty_print(&tokens, &reader));
    }
}

// print a Vec<InternedToken> in a nice way to check if the lexer is working
fn pretty_print(tokens: &[Token], reader: &RodeoReader) -> String {
    let mut out = String::new();

    for token in tokens {
        match token.data() {
            TokenData::None => {
                // If the token is Error print it in red using the colored crate
                out.push_str(&match token.kind() {
                    TokenKind::Error => reader.resolve(&token.spur()).red().to_string(),
                    TokenKind::Comment => reader.resolve(&token.spur()).dimmed().to_string(),
                    TokenKind::Whitespace
                    | TokenKind::Substitution
                    | TokenKind::Generation
                    | TokenKind::Execution
                    | TokenKind::LParen
                    | TokenKind::RParen
                    | TokenKind::LBracket
                    | TokenKind::RBracket
                    | TokenKind::Semicolon => reader.resolve(&token.spur()).to_string(),
                    TokenKind::UnnamedByte | TokenKind::UnnamedQuotation => {
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
                    TokenKind::String => format!("\"{}\"", reader.resolve(s)).green(),
                    TokenKind::RawString => format!("\"{}\"", reader.resolve(s)).green(),
                    TokenKind::Brainfuck => format!("`{}`", reader.resolve(s)).yellow(),
                    TokenKind::MacroInput => format!("{{{}}}", reader.resolve(s)).yellow(),
                    TokenKind::NamedByte | TokenKind::NamedQuotation => {
                        reader.resolve(s).cyan().bold()
                    }
                    TokenKind::Identifier => reader.resolve(s).cyan(),
                    _ => unreachable!(),
                }
                .to_string();

                out += &s;
            }
        }
    }

    out
}

fn debug_print(tokens: &[Token], reader: &RodeoReader) -> String {
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

pub fn parse_debug(file: Option<String>, bench: bool, debug: Option<bool>) {
    let file = file.unwrap_or(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap() + "/../libraries/std.sero")
            .to_str()
            .unwrap()
            .to_string(),
    );

    let debug = debug.unwrap_or(false);

    let content = std::fs::read_to_string(file).unwrap();

    let mut files = SimpleFiles::new();
    let file_id = files.add("std", &content);
    // create a span over the whole file
    let span = Span::from_range(0..content.len(), file_id);

    let mut rodeo = lasso::Rodeo::default();

    let (tokens, errors) = lex(&content, file_id, &mut rodeo);

    // Emit errors
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    // stop if there are errors
    if !errors.is_empty() {
        for error in errors {
            let diagnostic: Diagnostic<usize> = error.into();

            term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
        }
    }

    // Parse
    match parse_module(&tokens, span, rodeo.get_or_intern("std")) {
        Ok((module, warnings)) => {
            if bench {
                return;
            }

            if debug {
                println!("{:#?}", module);
            } else {
                println!("{:?}", module);
            }

            for warning in warnings {
                term::emit(&mut writer.lock(), &config, &files, &warning).unwrap();
            }
        }
        Err(error) => {
            let diagnostic: Diagnostic<usize> = error.into();

            term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
        }
    }
}
