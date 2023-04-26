# serotonin

:warning: currently working on a complete rewrite using a different set of technologies and better programming practices

A _toy_ [stack-oriented](https://en.wikipedia.org/wiki/Stack-oriented_programming) programming language that transpiles to [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) and is inspired by [Joy](https://hypercubed.github.io/joy/joy.html).

The project and documentation are works in progress.

## Hello World

```serotonin
IMPORT std;

main == "Hello, World!" sprint;
```

Compile:

```text
$ serotonin hello.sero
>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>+++++++++++++++++++++++++++++++++[<]>[.>]
```

## [Description](./description.md)

Check it out, and feel free to leave feedback.

## License

Copyright (c) 2022 Christopher Mahoney. This software is licensed under the [MIT license](https://opensource.org/licenses/MIT). Read [LICENSE](LICENSE) for all the details.
