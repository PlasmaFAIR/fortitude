# Fortitude

A Fortran linter and formatter, written in Rust.

Currently a work-in-progress, and is being tested using `test.f90`.

## TODO

- Attach methods to rules, access rules from a registry.
- Improved `use_modules` rules.
- Require `implicit none` rule.
- Avoid `double precision` rule.
- Prefer lower case rule.
- Consistent whitespace rules.
- Command line interface.
- Python package (see how ruff does it, uses `maturin`).
- Publish to `crates.io` and PyPI.
