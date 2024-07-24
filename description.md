# Language Description

Serotonin is a _toy_ stacked-based programming language designed to compile to BrainFuck. It is inspired by Joy and I attempt to take advantage of the happiness you find in declarative programming languages in the BrainFuck world. In contrast to that idea, I am not interested in hiding away the details of BrainFuck, both languages are very limiting.

This document goes into the background to motivate the existence of the language.

## Background

### Rewriting System

At the core of Serotonin is its rewriting system. Rather than preforming any sophisticated intermediate representation Serotonin recursively _rewrites_ functions with pre-defined alternatives that bring the final output closer to BrainFuck.

Mathematics is built on top of rewriting systems, grade school arithmetic is an example. Take for example the expression:

```text
3 * (5 + 2)
```

We can 'simplify' (reduce) this expression using common rewriting rules.

| expression      | rule                           |
|-----------------|--------------------------------|
| 3 * (5 + 6)     | 5 + 6 = 11                     |
| 3 * (11)        | (x) = x                        |
| 3 * 11          | 3 * 11 = 33                    |
| 33              | done                           |

Another such simplification is:

| expression      | rule                           |
|-----------------|--------------------------------|
| 3 * (5 + 6)     | a \* (b + c) = a \* b + a \* c |
| 3 \* 5 + 3 \* 6 | 3 * 5 = 15                     |
| 15 + 3 * 2      | 3 * 6 = 18                     |
| 15 + 6          | 15 + 18 = 33                   |
| 33              | done                           |

Serotonin is also backed by a rewriting system where our all our functions (operators) eventually get rewritten into BrainFuck code.

```sero
# 3 * (5 + 6)
main == 3 5 6 + * print;
# 3 == `>+++`
main == `>+++` 5 6 + * print;
# 5 == `>+++++`
main == `>+++` `>+++++` 6 + * print;
# 6 == `>++++++`
main == `>+++` `>+++++` `>++++++` + * print;
# + == `[-<+>]<`
main == `>+++` `>+++++` `>++++++` `[-<+>]<` * print;
# * == `<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<`;
main == `>+++` `>+++++` `>++++++` `[-<+>]<` `<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<` print;
# print == `.`
main == `>+++` `>+++++` `>++++++` `[-<+>]<` `<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<` `.`;
# concatenate
main == `>+++>+++++>++++++[-<+>]<<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<.`;
```

Serotonin is syntax and rules for creating a rewriting system, that when executed, produces BrainFuck programs.

### Postfix Notation

In the Serotonin program, you may have noticed that we wrote `3 * (5 + 6)` as `3 5 6 + *`. This is known as "postfix notation," in contrast to the more familiar "infix notation." The term "postfix" comes from Latin, meaning "after," indicating that operators are placed after their operands. In contrast, infix notation places operators between operands (prefix notation would be `* 3 + 5 6`). The use of postfix notation is a key feature of stack-based programming languages. Firstly, all expressions can be represented unambiguously without parentheses, as there is no concept of operator precedence. Anyone familiar with compilers can attest that handling precedence is cumbersome. Secondly, postfix expressions are directly executable by stack machines: numbers are pushed onto the stack, and operators pop operands off the stack before pushing the results back on.

| Stack | Instructions | Next Function |
|-------|--------------|---------------|
| empty | `3 5 6 + *`  | push 3        |
| 3     | `5 6 + *`    | push 5        |
| 3 5   | `6 + *`      | push 6        |
| 3 5 6 | `+ *`        | add           |
| 3 11  | `*`          | multiply      |

### But why stacks?

TODO

## Serotonin Language

Serotonin has a few key features that make it unique. In addition to the simple substitution rewriting rules Serotonin has compile-time meta-programming and execution rewriting rules, allowing for complex optimizations and control flow. To support these features, Serotonin has a concept of 'constraints' that allow us to implement functions with multiple rules depending on the context.

### Compile-time Meta-programming

There are 3 kinds of rewriting rules in Serotonin: "Substitution", "Generation" (meta-programming), and "Execution" (const-expr). Initial implementations of Serotonin only had substitution rules, as they are the most necessary. However, as I worked on the standard library I learned that BrainFuck programs can optimize very well when we're able to make assertions. Take for example `+`, the classic algorithm `[-<+>]<` requires a loop that iteratively decrements the top of stack and increments the element under, until the top of stack is zero. These loops are expensive and become much worse when considering `*` or `dupn`.

Generation rules are used to create optimized implementations of  

| Rewrite      | Symbol | Meaning                                                                                            |
|:-------------|:-------|:---------------------------------------------------------------------------------------------------|
| Substitution | `==`   | Replaces the term on the left with the terms on the right.                                         |
| Generation   | `==?`  | Executes the right-hand side, then replaces the left with the result treated as a BrainFuck block. |
| Execution    | `==!`  | Executes the right-hand side, then replaces the left with the result treated as constant bytes.    |

For example, the standard library addition function has the following rewrite rules.

```text
# + (a b -- a+b)
+ == `[-<+>]<`; # Substitution
+ (b) ==? '+' b dupn sprint; # Generation
+ (a b) ==! a b + pop; # Execution
```

### Constraints

Without constraints, every function could only have a single rule, and functions like `while` would be impossible to write. Constraints define special case rules for functions that can be applied via monomorphization. Constraints are often used when writing quality libraries. Here is the current list of available constraints:

| Constraint | Meaning |
|--------|---------|
| Lower Case Ascii. ie `a` | a byte we've named. we can match against |
| `@`                      | a byte we don't care to name |
| Number. ie `0`           | a byte that perfectly matches the number |
| Upper Case Ascii. ie `S` | a quotation we've given a name. we could match against |
| `?`                      | a quotation we haven't named |
| `[...]`                  | a quotation that is equivalent to what is between the braces |

Here are some examples of how to use constraints. Remember, the least preferred rule is written first. You may want to read bottom to top.

```text
true == 1;
false == 0;

# eq (a b -- a==b)
eq == `<[->-<]+>[<->[-]]<`;
eq (a b) == false;
eq (a a) == true;
eq (0) == zeq;         
# this says "read 0 eq => read zeq"

# zeq (a -- a==0)
zeq == `>+<[>[-]<[-]]>[-<+>]<`;
zeq (@) == false;
zeq (0) == true;

# {condition}[{body}{condition}]
while (C B) ==? C '[' sprint B sprint C ']<' sprint; 
while ([true] B) ==? '[' B ']<' sprint;
while ([false] ?) == ; # dead code elimination !
```

### Substitution

The first rewrite rule for addition is the most generalized form. It always works even if we don't know anything about the operands. This program would use substitution:

```text
main == read read + pop;
```

Substitution doesn't always have to be used without constraints. There are many places where it is nice to add additional constraints. Consider the `dup` function, with the following rules:

```text
# dup (a -- a a)
dup == `[->>+<<]>>[-<+<+>>]<`;
dup (a) == a a;
```

So this program `main == 10 dup + print;` could be rewritten as `main == 10 10 + print;`. Which generates faster code (at the expense of code size).

### Generation

The second rewrite for addition is an optimization. When the second operand is known at compile time we can create better code. Compare these two brainfuck programs:

```text
[ read | push 3 | add     ]
  ,      >+++     [-<+>]<
```

vs

```text
[ read | add 3]
  ,      +++
```

Both of these programs have the effect of reading a byte and then adding 3. However, everyone would agree that the second one is better. This pattern is common in brainfuck programming, we can often be clever if we have some constraints. The language wouldn't feel complete without a way to create these optimizations. To achieve this I took inspiration from Rust and Lisp, I want the entire language available at compile time.

So, a generation rule performs constraint substitution (kind of like "monomorphization" or "beta reduction"), gets compiled down to brianfuck, gets executed, and the resulting output is treated as brainfuck. This resulting program is inserted into a mangled substitution rule. Stepping through generation of `3 +` we see:

```text
+ (b) ==? '+' b dupn sprint;

+ (3) ==? '+' 3 dupn sprint; # monomorphization
+ (3) ==? '+++' sprint; # '+' 3 dupn becomes '+++' through some more nice rewriting 
+ (3) ==? '+++' `[<]>[.>]<` # sprint
+ (3) ==? `>+++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++` `[<]>[.>]<`
+ (3) ==? `>+++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++[<]>[.>]<`
# execute, returns [43, 43, 43] or +++

+ (3) == `+++`
```

Now "+ (3)" is a function with a mangled name that is available to the rest of the program. Its stack effect diagram is `a -- a+3`. In order to create control flow in Serotonin you use generation rules with composition constraints. Compositions are functions written between brackets. For example consider this replacement for the [yes](https://en.wikipedia.org/wiki/Yes_(Unix)) command:

```text
IMPORT std;

main == 'y' [true] [print] while;
```

`[true]` and `[print]` are quotations. They get compiled down to brainfuck code. Then the `while` function is able to use that code as a string to build more complicated programs.

### Execution

Execution is a lot like generation, but instead of the final substitution being a brainfuck block it is a non-terminated string. This allows for compile time execution of the program. In the case of addition, it allows us to rewrite:

```text
main == 2 2 + print;
```

as

```text
main == '\0x04' print;
```

Which compiles down to

```text
>++++.
```

Compared to without any optimization

```text
>++>++[-<+>]<.
```

## Macros

Earlier, when I wrote that there are 3 kinds of rewriting rules, I lied. There is a fourth rule, a "macro". Macros are similar to a generation rule but instead of executing Serotonin code to generate Brainfuck they execute Rust to generate Serotonin. Currently, there are no plans to include Macros as an extendable part of the language, they currently must be built into the compiler.

Macros have the following syntax:

```text
{ anything but curly braces! } name!
```

Within the compiler macros have the following signature: `fn(TODO) -> TODO`. They are given the `&str` between the curly braces and return Serotonin code to be substituted in place. Macros are helpful for more intense automated code generation that is too difficult to do in the current rewriting system. For example, the `autoperm!` macro is used to automatically generate programs to perform optimal tape shuffling using the [autoperm](https://github.com/Alextopher/autoperm) crate.

The `rot` function in the standard library is written as follows:

```text
# rot (a b c -- b c a)
rot == {a b c -- b c a} autoperm!;
rot (a b c) == b c a;
```

At the time of writing `{a b c -- b c a} autoperm!` generates the following BF block:

```text
`[->+<]<[->+<]<[->+<]>>>[-<<<+>>>]<`
```

There is not yet a known brainfuck program to do this.
