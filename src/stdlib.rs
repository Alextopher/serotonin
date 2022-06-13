use std::collections::HashMap;

use include_dir::{include_dir, Dir};

pub static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");

// create compositions
pub fn load_compositions<'a>() -> HashMap<&'a str, (usize, &'a str)> {
    // Build composition map
    let mut compositions = HashMap::new();

    compositions.insert("while", (2, "{0}[{1}{0}]<"));

    compositions
}

pub fn load_builtins<'a>() -> HashMap<&'a str, &'a str> {
    let mut builtins = HashMap::new();

    // dup (a -- a a)
    // copies the top of the stack
    // --------------------------------------------------
    //              - *a
    // [->+>+<<]  - *0  a  a
    // >>[-<<+>>]   -  a  a *0
    // <            -  a *a
    builtins.insert("dup", "[->+>+<<]>>[-<<+>>]<");

    // drop (a -- )
    // removes the top of the stack
    // --------------------------------------------------
    //              - *a
    // [-]          - *0
    // <            -
    builtins.insert("drop", "[-]<");

    // swap (a b -- b a)
    // swaps the top two elements of the stack
    // --------------------------------------------------
    //              -  a *b
    // [->>+<<]     - *0  b  a
    // >[-<+>]      -  b *0  a
    // >[-<+>]      -  b  a *0
    // <            -  b *a
    builtins.insert("swap", "<[->>+<<]>[-<+>]>[-<+>]<");

    // over (a b -- a b a)
    // --------------------------------------------------
    //                  -  a *b
    // <[->>+>+<<<]     - *0  b  a  a
    // >>>[-<<<+>>>]    -  a  b  a *0
    // <                -  a  b *a
    builtins.insert("over", "<[->>+>+<<<]>>>[-<<<+>>>]<");

    // rot (a b c -- b c a)
    // --------------------------------------------------
    //                  -  a  b *c
    // <<[->>>+<<<]     - *0  b  c  a
    // >[-<+>]          -  b *0  c  a
    // >[-<+>]          -  b  c *0  a
    // >[-<+>]          -  b  c  a *0
    // <                -  b  c *a
    builtins.insert("rot", "<<[->>>+<<<]>[-<+>]>[-<+>]>[-<+>]<");

    // print (a -- a)
    // prints a to stdout
    // --------------------------------------------------
    builtins.insert("print", ".");

    // read ( -- a)
    // reads a from the stdin
    // --------------------------------------------------
    builtins.insert("read", ">,");

    // inc (a -- a+1)
    // --------------------------------------------------
    builtins.insert("inc", "+");

    // dec (a -- a-1)
    // --------------------------------------------------
    builtins.insert("dec", "-");

    // + (a b -- a+b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<+>]           -  a+b *0
    // <                - *a+b
    builtins.insert("+", "[-<+>]<");

    // - (a b -- a-b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<->]           -  a-b *0
    // <                = *a-b
    builtins.insert("-", "[-<->]<");

    // eq (a b -- a==b)
    // returns 0 if a != b, 1 if a == b
    // x[-y-x]+y[x-y[-]]
    // https://esolangs.org/wiki/Brainfuck_algorithms#Wrapping_8
    // --------------------------------------------------
    builtins.insert("eq", "<[->-<]+>[<->[-]]<");

    // not (a -- !a)
    // temp0[-]+x[-temp0-]temp0[x+temp0-]
    // https://esolangs.org/wiki/Brainfuck_algorithms#x_.3D_not_x_.28boolean.2C_logical.29
    // --------------------------------------------------
    //                  - *a
    // >+<              - *a 1
    // [                - if a != 0
    //   [-]            - *0 1
    //   >-<            - *0 0
    // ]                - end if
    // >[<+>-]          - add temp to a
    // <                - *!a
    builtins.insert("not", ">+<[[-]>-<]>[<+>-]<");

    builtins
}


// Creates some lightly code golfed soultions 
pub fn generate_constants() -> Vec<String> {
    // 256 different constants
    let mut constants: Vec<String> = Vec::new();

    // add 0
    constants.push(String::from(""));
    for i in 1..=255 {
        // firstly generate the basic code
        // >{+}a[<{+}b>-]
        let (a, b) = get_best_factor(i);

        if a == 1 {
            constants.push(number_dumb(i));
        } else {
            let mut code = String::from(">");
            for _ in 0..a {
                code.push('+');
            }
            code.push_str("[<");
            for _ in 0..b {
                code.push('+');
            }
            code.push_str(">-]<");
            constants.push(code);
        }
    }
    // add 256 (0)
    constants.push(String::from(""));

    // Now we preform some optimizations
    loop {
        let mut updated = false;
        for i in 1..=255 {
            let len = constants[i].len();
            if constants[i - 1].len() + 1 < len {
                constants[i] =  "+".to_string() + &constants[i - 1];
                updated = true;
            }

            if constants[i + 1].len() + 1 < len {
                constants[i] = "-".to_string() + &constants[i + 1];
                updated = true;
            }
        }

        if !updated {
            break
        }
    }

    constants.iter().map(|block| format!(">{block}")).collect()
}

// generates the code to add a number to the stack
fn number_dumb<T>(n: T) -> String
where
    T: num::PrimInt + num::Unsigned,
{
    let mut result = String::from(">");

    num::range(T::zero(), n).for_each(|_| result.push_str("+"));

    result
}

// returns the two factors of n (a, b) such that a+b is minimized
fn get_best_factor(n: u16) -> (u16, u16) {
    let mut best_factor = (1, n);
    let mut best_sum = n + 1;

    for i in 2..=n {
        if n % i == 0 {
            let sum = i + n / i;
            if sum < best_sum {
                best_factor = (i, n / i);
                best_sum = sum;
            }
        }
    }

    best_factor
}
