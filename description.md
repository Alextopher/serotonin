# Langauge Description

Serotonin is a _toy_ stacked-based programming language designed to compile to Brainfuck. It is inspired by Joy and I attempt to take advantage of the happiness you find in declarative programming languages in the Brainfuck world. In contrast to that idea, I am not interested in hiding away the details of Brainfuck, both languages are very limiting.

## Rewriting system

At the core of Serotonin is its rewriting system. Rewriting systems are powerful ideas and are easy to understand. You likely already know a rewriting system: named algebra. Consider this expression:

```text
3 * (5 + 2)
```

We learn a set of rules we can apply to simplify this expression. One possible simplification is:

| expression  | rewrite | rule |
|-------------|---------|------|
| 3 * (5 + 2) | a * (b + c) = a \* b + a \* c | distributive property |
| 3 \* 5 + 3 \* 2 | 3 * 5 = 15 | multiplication |
| 15 + 3 * 2  | 3 * 2 = 6 | multiplication |
| 15 + 6      | 15 + 6 = 21 | addition |
| 21          | done

Another is

| expression  | rewrite | rule |
|-------------|---------|------|
| 3 * (5 + 2) | 5 + 2 = 7  | addition
| 3 * (7)     | (x) = x    | unwraping
| 3 * 7       | 3 * 7 = 21 | multiplication
| 21          | done

One way we can get into trouble with algebra is precedence. There is an agreed-upon order where we must prefer some rules over others. For example, rules that apply multiplication must be done before addition, and terms inside parentheses should be simplified first. In school, I was taught mnemonics to remember this order. **P**lease **E**xcuse **M**y **D**ear **A**unt **S**ally, parentheses - exponents - multiplication - division - addition - subtraction. Anyone who has learned algebra, or who has tried to write a compiler, knows precedence is tricky. It would be nice if we could remove almost all precedence rules while still being able to unambiguously simplify an expression.

## Postfix notation

Rather than inserting our operators between the operands we can make things simpler by putting the operators _after_ the operands. For example, our expression can be rewritten as:

```text
3 5 2 + *
```

This is a series of stack functions. Which is to say each function takes as input a stack and returns a stack. For example, `3` is a function that takes a stack `S` and returns `S 3`, or using a stack effect diagram we say `3` is this function:

```text
-- 3
```

Where the left-hand side of the `--` are the elements _poped_ from the stack and the right-hand side are the elements _pushed_ to the stack. You could think of this as the left having operands and the right having results. So addition could be `a b -- a+b`, where it takes 2 elements off that stack, the two terms, and it returns the sum.

To simplify this expression we only need to repeatedly execute each instruction.

| stack | instructions | next function |
|-------|--------------|------|
| empty | `3 5 2 + *`  | push 3
| 3     | `5 2 + *`    | push 5
| 3 5   | `2 + *`      | push 2
| 3 5 2 | `+ *`        | +
| 3 7   | `*`          | *
| 21    |              | done

Now to compile this expression to Brainfuck we define how to rewrite instructions (stack functions) to equivalent BF code.

| function    | equivlent brainfuck |
|-------------|---------------------|
| push 3      | >+++
| push 5      | >+++++
| push 2      | >++
| +           | [-<+>]<
| *           | <[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<

And now, using simple concatenation the resulting program is

```text
>+++>+++++>++[-<+>]<<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<
```

## Rewriting rules

In Serotonin, there are 3 kinds of rewriting rules. A single term can have multiple rewrite rules as long as they have independent _constraints_. Rewrites defined _last_ have higher precedence, assuming the constraints match. More on constraints later.

| Rewrite | symbol | meaning |
|:------|:--------|:---------|
| Subsitution | `==` | Replaces the term on the left with the terms on the right.
| Generation | `==?` | Executes the right-hand side, then replaces the left with the result treated as a Brainfuck block.
| Execution | `==!` | Executes the right-hand side, then replaces the left with the result treated as constant bytes.

For example, the standard library addition function has the following rewrite rules.

```text
# + (a b -- a+b)
+ == `[-<+>]<`; # Subsitution
+ (b) ==? '+' b dupn sprint; # Generation
+ (a b) ==! a b + pop; # Execution
```

### Subsitution

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

So this program `main == 10 dup + print;` could be rewriten as `main == 10 10 + print;`. Which generates faster code (at the expense of code size).

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

So, a generation rule performs constraint substitution (monomorphization), gets compiled down to brianfuck, gets executed, and the resulting output is treated as brainfuck. This resulting program is inserted into a mangled substitution rule. Stepping through generation of `3 +` we see:

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

## Constraints

Without constraints, every function could only have a single rule, and functions like `while` would be impossible to write. Constraints define special case rules for functions that can be applied via monomorphization. Constraints are often used when writing quality libraries. Here is the current list of available constraints:

| Constraint | Meaning |
|--------|---------|
| Lower Case Ascii. ie `a` | a byte we've named. we can match against
| `@`                      | a byte we don't care to name
| Number. ie `0`           | a byte that perfectly matches the number
| Upper Case Ascii. ie `S` | a quotation we've given a name. we could match against
| `?`                      | a quotation we haven't named
| `[...]`                  | a quotation that is equivalent to what is between the braces

Here are some examples of how to use constraints. Remember, the least preferred rule is written first. You may want to read bottom to top.

```text
# eq (a b -- a==b)
eq == `<[->-<]+>[<->[-]]<`;
eq (a b) == false;
eq (a a) == true;
eq (0) == zeq;

# zeq (a -- a==0)
zeq == `>+<[>[-]<[-]]>[-<+>]<`;
zeq (@) == false;
zeq (0) == true;

# {condition}[{body}{condition}]
while (C B) ==? C '[' sprint B sprint C ']<' sprint; 
while ([true] B) ==? '[' B ']<' sprint;
while ([false] ?) ==? ; # dead code elimination !
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

There is no known brainfuck program to do this.
