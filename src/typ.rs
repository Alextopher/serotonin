// Serotonin uses a type system heavily inspired by https://prl.ccs.neu.edu/blog/static/stack-languages-talk-notes.pdf

#[derive(Debug)]
pub struct StackEffect {
    i: u32,
    o: u32,
}

// Compositions of StackEffects as defined in the HPCL section of the reference
// Since there is only 1 type it is impossible to create invalid states and
// rather than keeping lists of types we just keep the count
// stack effect `b b b b -> b b` becomes `4 -> 2`
fn composition(s1: &StackEffect, s2: &StackEffect) -> StackEffect {
    if s1.o == 0 {
        return StackEffect {
            i: s1.i + s2.i,
            o: s2.o,
        };
    }

    if s2.i == 0 {
        return StackEffect {
            i: s1.i,
            o: s1.o + s2.o,
        };
    }

    composition(
        &StackEffect {
            i: s1.i,
            o: s1.o - 1,
        },
        &StackEffect {
            i: s2.i - 1,
            o: s2.o,
        },
    )
}

// 3. Conditional Branching Language
// Taking a page on how non-deterministic-finite-automatas types are actually sets of "possible" StackEffects
// Composing two types by composing each element of one type with each element in the other
//
// During type inference we know that if there is only 1 stack effect left in the set then we know the type
//
// Some functions have infinite sets of valid stack effects, for example:
//
// main == "hello" [] [pop] while;
// shortened to:
// main == "hello" spop;
//
// "hello" pushes 6 bytes to the stack (the string "hello" and a null byte) spop removes bytes off the stack until it is empty.
// the type of spop must be {n:0 | n > 0}. I represent this stack effect as ?:0. We may also consider sread
//
// sread == `,[>,]`
//
// sread pushes bytes from input until a null byte is read. Since there is no way to know the number of bytes that will be read it should be of type 0:?
