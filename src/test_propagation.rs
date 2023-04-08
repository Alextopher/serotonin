// equivalent("1 2 swap", "2 1");
macro_rules! equivalent {
    ($a:expr, $b:expr) => {
        assert_eq!(
            compile(
                "main",
                &format!("IMPORT std; main == {};", $a),
                Config::default()
            ),
            compile(
                "main",
                &format!("IMPORT std; main == {};", $b),
                Config::default()
            )
        );
    };
}

macro_rules! fails {
    ($a:expr) => {
        assert!(compile("main", $a, Config::default()).is_err())
    };
}

// ints match corresponding hex
#[test]
fn hex_matches_int() {
    for i in 0u8..=255 {
        equivalent!(format!("{:#04x}", i), format!("{}", i));
    }
}

// STACK MANIPULATION
#[test]
fn stack_manipulation() {
    equivalent!("1 dup", "1 1");
    equivalent!("1 2 dup2", "1 2 1 2");

    equivalent!("1 drop", "");
    equivalent!("1 2 drop2", "");

    equivalent!("1 2 swap", "2 1");
    equivalent!("1 2 3 4 swap2", "3 4 1 2");

    equivalent!("1 2 over", "1 2 1");
    equivalent!("1 2 3 4 over2", "1 2 3 4 1 2");

    equivalent!("1 2 3 rot", "2 3 1");
    equivalent!("1 2 3 rot rot rot", "1 2 3");
    equivalent!("1 2 3 4 5 6 rot2", "3 4 5 6 1 2");
    equivalent!("1 2 3 4 5 6 rot2 rot2 rot2", "1 2 3 4 5 6");

    equivalent!("1 2 3 -rot", "3 1 2");
    equivalent!("1 2 3 -rot -rot -rot", "1 2 3");
    equivalent!("1 2 3 4 5 6 -rot2", "5 6 1 2 3 4");
    equivalent!("1 2 3 4 5 6 -rot2 -rot2 -rot2", "1 2 3 4 5 6");

    equivalent!("1 2 nip", "2");
    equivalent!("1 2 3 4 nip2", "3 4");

    equivalent!("1 2 tuck", "2 1 2");
    equivalent!("1 2 3 4 tuck2", "3 4 1 2 3 4");
}

// ARITHMETIC
#[test]
fn arithmetic() {
    // inc
    equivalent!("40 inc", "41");
    equivalent!("0 inc", "1");
    equivalent!("255 inc", "0");

    // dec
    equivalent!("40 dec", "39");
    equivalent!("0 dec", "255");
    equivalent!("1 dec", "0");

    // +
    equivalent!("40 2 +", "42");
    equivalent!("0 0 +", "0");
    equivalent!("255 1 +", "0");
    equivalent!("255 2 +", "1");

    // -
    equivalent!("40 2 -", "38");
    equivalent!("0 0 -", "0");
    equivalent!("0 1 -", "255");
    equivalent!("1 2 -", "255");

    // *
    equivalent!("40 2 *", "80");
    equivalent!("0 0 *", "0");
    equivalent!("0 1 *", "0");
    equivalent!("1 0 *", "0");
    equivalent!("255 1 *", "255");
    equivalent!("255 2 *", "254");
}

#[test]
fn dupn() {
    // dupn
    equivalent!("1 10 dupn", "1 1 1 1 1 1 1 1 1 1");
    equivalent!("0 10 dupn", "0 0 0 0 0 0 0 0 0 0");
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
