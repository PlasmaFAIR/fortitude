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

## Does fortitude handle fixed-form Fortran?

Not currently. Fortitude's parser and rules are intended for free-form
Fortran source. Fixed-form source files, such as files commonly using
`.f`, `.for`, or `.ftn` extensions, are not included in Fortitude's
default file discovery patterns.

You can pass fixed-form files explicitly or add their extensions to the
[`include`](settings.md#include) setting, but Fortitude will still parse
them as free-form source. This may produce syntax errors or misleading
diagnostics, so fixed-form projects are not supported yet.

## What is "preview"?

Preview enables a collection of newer rules and fixes that are considered experimental or unstable.
See the [preview documentation](preview.md) for more details; or, to see which rules are currently
in preview, visit the [rules reference](rules.md).
