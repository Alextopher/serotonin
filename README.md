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
| pop       | `(a -- )`             | print the top item on the stack and remove it
| read      | `( -- a)`             | read a character from the input stream
| inc       | `(a -- a+1)`          | increment the top item on the stack
| dec       | `(a -- a-1)`          | decrement the top item on the stack
| +         | `(a b -- a+b)`        | add the top two items on the stack
| -         | `(a b -- a-b)`        | subtract the top two items on the stack