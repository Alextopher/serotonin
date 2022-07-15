// Optimizes generated Brainfuck code
// - The compiler likes to generate code like ">program<". Remove the unnecessary moves.
// - Many times we end up having code with unneccessary neighboring moves "<>>" simplify them.
pub fn optimize_bf(mut bf: String) -> String {
    // if the first character is ">" and the last character is "<", remove the first and last character
    if bf.starts_with(">") && bf.ends_with("<") {
        bf.remove(0);
        bf.remove(bf.len() - 1);
    }

    let mut acc = vec![];
    for c in bf.chars() {
        match (acc.last(), c) {
            (Some('>'), '<') => {
                acc.pop();
            }
            (Some('<'), '>') => {
                acc.pop();
            }
            _ => {
                acc.push(c);
            }
        }
    }

    acc.iter().collect()
}
