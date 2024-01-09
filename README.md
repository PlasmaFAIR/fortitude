# Fortitude

A Fortran linter and formatter, written in Rust. The obvious name 'Flint' was taken (by
multiple projects).

Currently a work-in-progress, and is being tested using `test.f90`:

```bash
$ cargo run test.f90
```

Can also run unit tests:

```bash
$ cargo test
```

## TODO

- Report `implicit none` use in functions when its already in an enclosing module
- Avoid `double precision`.
- Rule for `use module` without an `only` clause.
- Rework relationship between rules and methods. Instead of rules owning a method,
  there should be a map between rules and methods (and also between rule code strings
  and rules).
- Rework how rule codes are shared between rules and violations. Violations should not
  have a rule code, and this should be inserted by the rule itself.
- Command line interface.
  - Allow users to switch rules on and off via `--include` and `--exclude`.
  - Use `.fortitude.toml` file to set rules project wide.
  - Work on multiple files.
- Syntax error rule (just scan the tree and report all error nodes)
- Code style rules
- After gathering violations, check per-file and per-line ignores and discard those we
  don't care about.
- Install executable
- Python package (see how ruff does it, use `maturin`).
- Publish to `crates.io` and PyPI.

## Wishlist

- Report if a function can be marked pure.
- Report unused variables.
- Report things like `real(8)`, and recommend using `real64` from `iso_fortran_env`.
  To do this properly, will need to look up through relevant scopes to figure out if the
  user has defined something like `dp`.

## Contributing

Please feel free to add or suggest new rules, or comment on the layout of the project
while it's still at this early stage of development. When contributing, please use
`cargo clippy` to lint your code, and `cargo fmt` to format it.
