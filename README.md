# Fortitude

A Fortran linter and formatter, written in Rust. Currently a work-in-progress.

To see the available commands:

```bash
$ cargo run
```

To lint some files:

```bash
$ cargo run check [FILES]
```

By default, this will lint all `.f90` files from your current working directory.

## TODO

- Command line interface.
    - `check` mode
      - Use `.fortitude.toml` file to set rules project wide.
      - Better error reporting when given unknown rules
  - `explain` mode, print help text for a given rule
- Propagate rule errors
- Syntax error rule (just scan the tree and report all error nodes)
- A few code style rules (leave most until after initial release)
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
