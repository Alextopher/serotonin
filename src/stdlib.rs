use std::collections::HashMap;

use include_dir::{include_dir, Dir};

pub static LIBRARIES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/libraries");

// Build composition map
pub fn load_compositions<'a>() -> HashMap<&'a str, (usize, &'a str)> {
    let mut compositions = HashMap::new();

    compositions.insert("while", (2, "{0}[{1}{0}]<"));

    compositions
}

// Creates some lightly code golfed soultions
// The "general" idea of these solutions is to represent a number as a*b+c
// such that a+b+|c| is minimized
pub fn load_code_golfed_constants() -> Vec<String> {
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
                constants[i] = "+".to_string() + &constants[i - 1];
                updated = true;
            }

            if constants[i + 1].len() + 1 < len {
                constants[i] = "-".to_string() + &constants[i + 1];
                updated = true;
            }
        }

        if !updated {
            break;
        }
    }

    constants.iter().map(|block| format!(">{block}")).collect()
}

pub fn load_simple_constants() -> Vec<String> {
    let mut constants: Vec<String> = Vec::new();
    for i in 0..=255 {
        constants.push(number_dumb(i));
    }
    constants
}

// generates the code to add a number to the stack
fn number_dumb(n: u16) -> String {
    let mut result = String::from(">");

    for _ in 0..n {
        result.push('+');
    }

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
