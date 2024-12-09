# FAQ

## Does fortitude handle preprocessor files?

`fortitude` uses the
[tree-sitter](https://tree-sitter.github.io/tree-sitter/) [Fortran
grammar](https://github.com/stadelmanma/tree-sitter-fortran) to parse
your source code, and currently this has some support for the
preprocessor -- essentially limited to `#include`, and `#if` and
`#ifdef` guards around complete statements or constructs.

However, tree-sitter is fairly error tolerant, and `fortitude` should
still run fine, you might just get spurious syntax errors. You can
ignore these project wide with `--ignore=E001`.


## What is "preview"?

Preview enables a collection of newer rules and fixes that are considered experimental or unstable.
See the [preview documentation](preview.md) for more details; or, to see which rules are currently
in preview, visit the [rules reference](rules.md).
