use crate::compile;

// equivalent("1 2 swap", "2 1");
macro_rules! equivalent {
    ($a:expr, $b:expr) => {
        assert_eq!(
            compile("main", &format!("IMPORT std; main == {};", $a)),
            compile("main", &format!("IMPORT std; main == {};", $b))
        );
    };
}

macro_rules! exact {
    ($a:expr, $b:expr) => {
        assert_eq!(
            compile("main", &format!("IMPORT std; main == {};", $a)).unwrap(),
            $b
        )
    };
}

macro_rules! fails {
    ($a:expr) => {
        assert!(compile("main", $a).is_err())
    };
}

// STACK MANIPULATION
#[test]
fn stack_manipulation() {
    // swap
    equivalent!("1 2 swap", "2 1");
    equivalent!("0 0 swap", "0 0");
    equivalent!("0 1 swap", "1 0");
    equivalent!("1 0 swap", "0 1");

    // dup
    equivalent!("1 dup", "1 1");
    equivalent!("0 dup", "0 0");
    equivalent!("0 1 dup", "0 1 1");
    equivalent!("1 0 dup", "1 0 0");

    // drop
    equivalent!("1 drop", "");
    equivalent!("0 drop", "");
    equivalent!("0 1 drop", "0");
    equivalent!("1 0 drop", "1");

    // over
    equivalent!("1 2 over", "1 2 1");
    equivalent!("0 0 over", "0 0 0");
    equivalent!("0 1 over", "0 1 0");
    equivalent!("1 0 over", "1 0 1");

    // rot
    equivalent!("1 2 3 rot", "2 3 1");
    equivalent!("0 0 0 rot", "0 0 0");
    equivalent!("0 1 0 rot", "1 0 0");
    equivalent!("1 0 1 rot", "0 1 1");
    equivalent!("1 2 3 4 rot", "1 3 4 2");
}

// ARITHMETIC
#[test]
fn arithmetic() {
    // inc
    equivalent!("40 inc", "41");
    equivalent!("0 inc", "1");
    equivalent!("255 inc", "0");
    exact!("read inc", ">,+");

    // dec
    equivalent!("40 dec", "39");
    equivalent!("0 dec", "255");
    equivalent!("1 dec", "0");
    exact!("read dec", ">,-");

    // +
    equivalent!("40 2 +", "42");
    equivalent!("0 0 +", "0");
    equivalent!("255 1 +", "0");
    equivalent!("255 2 +", "1");
    exact!("read 1 +", ">,+");

    // -
    equivalent!("40 2 -", "38");
    equivalent!("0 0 -", "0");
    equivalent!("0 1 -", "255");
    equivalent!("1 2 -", "255");
    exact!("read 1 -", ">,-");
    exact!("read 20 -", ">,--------------------");

    // *
    equivalent!("40 2 *", "80");
    equivalent!("0 0 *", "0");
    equivalent!("0 1 *", "0");
    equivalent!("1 0 *", "0");
    equivalent!("255 1 *", "255");
    equivalent!("255 2 *", "254");
    exact!("read 10 *", ",[->++++++++++<]>[-<+>]")
}

#[test]
fn dupn() {
    // dupn
    equivalent!("1 10 dupn", "1 1 1 1 1 1 1 1 1 1");
    equivalent!("0 10 dupn", "0 0 0 0 0 0 0 0 0 0");
    exact!(
        "read 10 dupn",
        ">,[->+>+>+>+>+>+>+>+>+>+<<<<<<<<<<]>[>]<[-<<<<<<<<<<+>>>>>>>>>>]"
    );
}

// LOGIC
#[test]
fn logic() {
    // true & false
    equivalent!("true", "1");
    equivalent!("false", "0");

    // not
    equivalent!("true not", "false");
    equivalent!("false not", "true");
    equivalent!("20 not", "0");

    // eq
    equivalent!("1 1 eq", "true");
    equivalent!("2 1 eq", "false");

    // neq
    equivalent!("1 1 neq", "false");
    equivalent!("2 1 neq", "true");

    // zeq
    equivalent!("0 zeq", "true");
    equivalent!("1 zeq", "false");
    equivalent!("255 zeq", "false");

    // 0 eq becomes zeq
    equivalent!("read 0 eq", "read zeq");
}

#[test]
fn recursion_fails() {
    fails!("main == fn; fn == main;");
    fails!("main == 10 20 fn; fn (a b) == a b fn;");
}
