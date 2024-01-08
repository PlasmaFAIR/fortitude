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
- Require `implicit none` in interface functions
- Avoid `double precision`.
- Code style rules
- Command line interface.
  - Allow users to switch rules on and off by their code.
  - Use `.fortitude.toml` file to set project wide defaults
  - Work on multiple files.
- After gathering violations, check per-file and per-line ignores and discard those we
  don't care about.
- Implement helper functions to reduce repeat code in per-rule tests.
- Install executable
- Python package (see how ruff does it, use `maturin`).
- Publish to `crates.io` and PyPI.

## Contributing

Please feel free to add or suggest new rules, or comment on the layout of the project
while it's still at this early stage of development. When contributing, please use
`cargo clippy` to lint your code, and `cargo fmt` to format it.
