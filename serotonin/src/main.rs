mod debug;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    bench: bool,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Debug the lexer
    Lexer {
        #[arg(short, long)]
        file: Option<String>,

        #[arg(short, long)]
        debug: Option<bool>,
    },
    /// Debug the parser
    Parser {
        #[arg(short, long)]
        file: Option<String>,

        #[arg(short, long)]
        debug: Option<bool>,
    },
}

fn main() {
    let args = Cli::parse();

    match args.subcommand {
        Some(Commands::Lexer { file, debug }) => debug::lex_debug(file, args.bench, debug),
        Some(Commands::Parser { file, debug }) => debug::parse_debug(file, args.bench, debug),
        None => println!("No subcommand was used"),
    }
}
