use std::collections::HashMap;

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