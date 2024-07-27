/// The lexer creates a stream of tokens from a file or string.
mod lexer;
/// The parser transforms a stream of tokens into an abstract syntax tree.
mod parser;
/// The semantic analyzer checks the AST for errors and creates a symbol table.
mod semantic;

pub use lexer::{lex, InternedToken, Span, Token, TokenData, TokenKind};
pub use parser::{ast, parse_definition, parse_module};
pub use semantic::SemanticAnalyzer;

pub(crate) const ICE_NOTE: &str =
    "This is a compiler error and should not have happened. Please report this as a bug.";

/// Utility method that generates random (syntactically valid) BrainFuck programs.
#[cfg(test)]
pub(crate) fn random_brainfuck(n: usize) -> String {
    use std::cmp::Ordering;

    use rand::Rng;

    let mut rng = rand::thread_rng();
    let commands = ['+', '-', '>', '<', '.', ','];
    let mut program = String::with_capacity(n);

    // Track the number of unmatched '['
    let mut open_brackets = 0;

    for _ in 0..n {
        let cmd = if open_brackets == 0 {
            // Cannot insert a ']' if there are no unmatched '['
            let cmd_index = rng.gen_range(0..commands.len() + 1);
            if cmd_index < commands.len() {
                commands[cmd_index]
            } else {
                '['
            }
        } else {
            // Can insert any command, including '[' and ']'
            let cmd_index = rng.gen_range(0..commands.len() + 2);
            match cmd_index.cmp(&(commands.len())) {
                Ordering::Less => commands[cmd_index],
                Ordering::Equal => '[',
                Ordering::Greater => ']',
            }
        };

        match cmd {
            '[' => open_brackets += 1,
            ']' => {
                if open_brackets > 0 {
                    open_brackets -= 1;
                } else {
                    unreachable!()
                }
            }
            _ => {}
        }

        program.push(cmd);
    }

    // If there are unmatched '[' at the end, add the matching ']'
    for _ in 0..open_brackets {
        program.push(']');
    }

    program
}
