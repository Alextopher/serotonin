use std::collections::HashMap;

use crate::parse::Ty;

// Read src/lib.joy into a string at compile time
pub(crate) static LIBRARY: &'static str = include_str!("lib.joy");

fn def(pops: i64, pushes: i64, code: &str) -> (String, Ty) {
    (code.into(), Ty { pops, pushes })
}

// create hashmap of standard library functions
pub fn builtin() -> HashMap<String, (String, Ty)> {
    let mut stdlib = HashMap::new();

    // dup (a -- a a)
    // copies the top of the stack
    // --------------------------------------------------
    //              - *a
    // >[-]>[-]     -  a  0 *0
    // <<[->+>+<<]  - *0  a  a
    // >>[-<<+>>]   -  a  a *0
    // <            -  a *a
    stdlib.insert("dup", def(1, 2, ">[-]>[-]<<[->+>+<<]>>[-<<+>>]<"));

    // drop (a -- )
    // removes the top of the stack
    // --------------------------------------------------
    //              - *a
    // [-]          - *0
    // <            -
    stdlib.insert("drop", def(1, 0, "[-]<"));

    // swap (a b -- b a)
    // swaps the top two elements of the stack
    // --------------------------------------------------
    //              -  a *b
    // >[-]         -  a  b *0 - TODO ensure above the stack is clean
    // <<[->>+<<]   - *0  b  a
    // >[-<+>]      -  b *0  a
    // >[-<+>]      -  b  a *0
    // <            -  b *a
    stdlib.insert("swap", def(2, 2, ">[-]<<[->>+<<]>[-<+>]>[-<+>]<"));

    // over (a b -- a b a)
    // --------------------------------------------------
    //                  -  a *b
    // >[-]>[-]         -  a  b  0 *0 - TODO ensure above the stack is clean
    // <<<[->>+>+<<<]   - *0  b  a  a
    // >>>[-<<<+>>>]    -  a  b  a *0
    // <                -  a  b *a
    stdlib.insert("over", def(2, 3, ">[-]>[-]<<<[->>+>+<<<]>>>[-<<<+>>>]<"));

    // rot (a b c -- b c a)
    // --------------------------------------------------
    //                  -  a  b *c
    // >[-]             -  a  b  c *0 - TODO ensure above the stack is clean
    // <<<[->>>+<<<]    - *0  b  c  a
    // >[-<+>]          -  b *0  c  a
    // >[-<+>]          -  b  c *0  a
    // >[-<+>]          -  b  c  a *0
    // <                -  b  c *a
    stdlib.insert("rot",  def(3, 3, ">[-]<<<[->>>+<<<]>[-<+>]>[-<+>]>[-<+>]<"));

    // print (a -- a)
    // prints a to stdout
    // --------------------------------------------------
    stdlib.insert("print", def(1, 1, "."));

    // read ( -- a)
    // reads a from the stdin
    // --------------------------------------------------
    stdlib.insert("read", def(0, 1, ">,"));

    // inc (a -- a+1)
    // --------------------------------------------------
    stdlib.insert("inc", def(1,1,"+"));

    // dec (a -- a-1)
    // --------------------------------------------------
    stdlib.insert("dec", def(1,1,"-"));

    // + (a b -- a+b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<+>]           -  a+b *0
    // <                - *a+b
    stdlib.insert("+", def(2, 1, "[-<+>]<"));

    // - (a b -- a-b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<->]           -  a-b *0
    // <                = *a-b
    stdlib.insert("-", def(2, 1, "[-<->]<"));

    // eq (a b -- a==b)
    // returns 0 if a != b, 1 if a == b
    // x[-y-x]+y[x-y[-]]
    // https://esolangs.org/wiki/Brainfuck_algorithms#Wrapping_8
    // --------------------------------------------------
    stdlib.insert("eq", def(2, 1, "<[->-<]+>[<->[-]]<"));

    // not (a -- !a)
    // temp0[-]+x[-temp0-]temp0[x+temp0-]
    // https://esolangs.org/wiki/Brainfuck_algorithms#x_.3D_not_x_.28boolean.2C_logical.29
    // --------------------------------------------------
    //                  - *a
    // >[-]+<           - *a 1
    // [                - if a != 0
    //   [-]            - *0 1
    //   >-<            - *0 0
    // ]                - end if
    // >[<+>-]          - add temp to a
    // <                - *!a
    stdlib.insert("not", def(1, 1, ">[-]+<[[-]>-<]>[<+>-]<"));

    // shift (? -- a)
    // this is an unsafe operator and it just shifts the stack up by one
    // useful for testing but USE WITH CAUTION
    // --------------------------------------------------
    stdlib.insert("shift", def(0, 1, ">"));

    // unshift (a b -- a ?)
    // this is an unsafe operator and it just shifts the stack down by one
    // unlike drop it does not remove the top element
    // useful for testing but USE WITH CAUTION
    // --------------------------------------------------
    stdlib.insert("unshift", def(1, 0, "<"));

    // create a String - String hashmap
    let mut string_lib = HashMap::new();
    for (key, value) in stdlib {
        string_lib.insert(key.to_string(), value);
    }
    string_lib
}
