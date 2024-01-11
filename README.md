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

- Avoid `real*8`.
- Rule for floating point number literals without a kind suffix.
- Rule for `use module` without an `only` clause.
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

The following will require better analysis of scope:

- Report if a function can be marked pure.
- Report unused variables.
- Report undefined variables.

## Contributing

Please feel free to add or suggest new rules, or comment on the layout of the project
while it's still at this early stage of development. When contributing, please use
`cargo clippy` to lint your code, and `cargo fmt` to format it.
