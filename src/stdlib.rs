use std::collections::HashMap;

use crate::{parse::{AstNode, parser}, gen};

// Read src/lib.joy into a string at compile time
static LIBRARY: &'static str = include_str!(concat!("lib.joy"));

// create hashmap of standard library functions
pub fn builtin() -> HashMap<String, String> {
    let mut stdlib = HashMap::new();

    // dup (a -- a a)
    // copies the top of the stack
    // --------------------------------------------------
    //              - *a
    // >[-]>[-]     -  a  0 *0
    // <<[->+>+<<]  - *0  a  a
    // >>[-<<+>>]   -  a  a *0
    // <            -  a *a
    stdlib.insert("dup", ">[-]>[-]<<[->+>+<<]>>[-<<+>>]<");

    // drop (a -- )
    // removes the top of the stack
    // --------------------------------------------------
    //              - *a
    // [-]          - *0
    // <            -
    stdlib.insert("drop", "[-]<");

    // swap (a b -- b a)
    // swaps the top two elements of the stack
    // --------------------------------------------------
    //              -  a *b
    // >[-]         -  a  b *0 - TODO ensure above the stack is clean
    // <<[->>+<<]   - *0  b  a
    // >[-<+>]      -  b *0  a
    // >[-<+>]      -  b  a *0
    // <            -  b *a
    stdlib.insert("swap", ">[-]<<[->>+<<]>[-<+>]>[-<+>]<");

    // over (a b -- a b a)
    // --------------------------------------------------
    //                  -  a *b
    // >[-]>[-]         -  a  b  0 *0 - TODO ensure above the stack is clean
    // <<<[->>+>+<<<]   - *0  b  a  a
    // >>>[-<<<+>>>]    -  a  b  a *0
    // <                -  a  b *a
    stdlib.insert("over", ">[-]>[-]<<<[->>+>+<<<]>>>[-<<<+>>>]<");

    // rot (a b c -- b c a)
    // --------------------------------------------------
    //                  -  a  b *c
    // >[-]             -  a  b  c *0 - TODO ensure above the stack is clean
    // <<<[->>>+<<<]    - *0  b  c  a
    // >[-<+>]          -  b *0  c  a
    // >[-<+>]          -  b  c *0  a
    // >[-<+>]          -  b  c  a *0
    // <                -  b  c *a
    stdlib.insert("rot", ">[-]<<<[->>>+<<<]>[-<+>]>[-<+>]>[-<+>]<");

    // print (a -- a)
    // prints a to stdout
    // --------------------------------------------------
    stdlib.insert("print", ".");

    // read ( -- a)
    // reads a from the stdin
    // --------------------------------------------------
    stdlib.insert("read", ">,");

    // inc (a -- a+1)
    // --------------------------------------------------
    stdlib.insert("inc", "+");

    // dec (a -- a-1)
    // --------------------------------------------------
    stdlib.insert("dec", "-");

    // + (a b -- a+b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<+>]           -  a+b *0
    // <                - *a+b
    stdlib.insert("+", "[-<+>]<");

    // - (a b -- a-b)
    // --------------------------------------------------
    //                  -    a *b
    // [-<->]           -  a-b *0
    // <                = *a-b
    stdlib.insert("-", "[-<->]<");

    // create a String - String hashmap
    let mut string_lib = HashMap::new();
    for (key, value) in stdlib {
        string_lib.insert(key.to_string(), value.to_string());
    }
    string_lib
}


// more complex functions are written using the bultins
pub fn load_lib(compiled: &mut HashMap<String, String>) {
    let root = parser(&LIBRARY);

    if let AstNode::Compound(_name, private, public) = root {
        // merge the private definitions into the definitions
        for definition in private {
            // compile the definition
            let name = &definition.get_name();
            let code = gen::gen_bf(definition, compiled);
            compiled.insert(name.to_string(), code);
        }

        // merge the public definitions into the definitions
        for definition in public {
            let name = &definition.get_name();
            let code = gen::gen_bf(definition, compiled);
            compiled.insert(name.to_string(), code);
        }
    }
}