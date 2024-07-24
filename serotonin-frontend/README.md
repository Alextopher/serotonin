# Serotonin Frontend

The frontend of the compiler has 3 components:

- Lexer
- Parser
- Semantic Analyzer

These have standard meanings in the context of compilers. Putting their actions explicitly:

- Lexer: `&str` -> `Vec<Token>`
- Parser: `Vec<Token>` -> `AST`
- Semantic Analyzer: `AST` -> `SymbolTable`
