# A programming language designed to compile to brainfuck


## Docs
| function  | stack effect          | description   
| :-------- | :-------------------- | :------------
| dup       | `(a -- a a)`          | duplicate the item on top of the stack
| drop      | `(a -- )`             | remove the item on top of the stack
| swap      | `(a b -- b a)`        | swap the top two items on the stack
| over      | `(a b -- a b a)`      | duplicate the second item on the stack
| rot       | `(a b c -- b c a)`    | rotate the top three items on the stack 
| print     | `(a -- a)`            | print the top item on the stack
| pop       | `(a -- )`             | `print drop`
| read      | `( -- a)`             | read a character from the input stream
| inc       | `(a -- a+1)`          | increment the top item on the stack
| dec       | `(a -- a-1)`          | decrement the top item on the stack
| +         | `(a b -- a+b)`        | add the top two items on the stack
| -         | `(a b -- a-b)`        | subtract the top two items on the stack
| todo *    | `(a b -- a*b)`        | multiply the top two items on the stack
| todo /    | `(a b -- a/b)`        | divide the top two items on the stack
| eq        | `(a b -- a == b)`     | compare the top two items on the stack. if equal return 1 otherwise return 0
| not       | `(a -- !a)`           | if a == 0 return 1 _otherwise_ return 0
| neq       | `(a b -- a != b)`     | compare the top two items on the stack. if not equal return 1 otherwise return 0
| shift     | `(a ? -- a b)`        | shift **unsafely** moves the stack pointer to the right by one
| unshift   | `(a b -- a ?)`        | unshift **unsafely** moves the stack pointer to the left by one

## control flow

### `[condition] [then] [else] ifte`
If "condition" is true execute "then" otherwise execute "else". Brainfuck is typically destructive, however the top of the stack is duplicated so that the branches have access to the original values.

```
read [eq 0] [pop] [dup + pop] ifte
```

```rust
if (input == 0) { // <- typically this would destory "input"
    println!("{}", input);
} else {
    println!("{}", input + input)
}
```

### `[condition] [code] while`
While "condition" is true execute "code". Brainfuck is typically destructive, however the top of the stack is duplicated so that the branches have access to the original values.

