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

and lint with clippy:

```bash
$ cargo clippy
```

## TODO

- `use_modules` rule for interfaces and other constructs that should be in modules.
- Require `implicit none` rule.
- Avoid `double precision` rule.
- Code style rules
- `Rule` should be const constructable.
- Rules should have status: `Default` or `Optional`
- Implement rule registry, ideally populated at compile time.
- Map rule code strings to `rules::Code`, e.g.
  `"B001" -> Code(Category::BestPractices, 1)`.
- Command line interface.
  - Allow users to switch rules on and off by their code.
  - Use `.fortitude.toml` file to set project wide defaults
  - Work on multiple files.
- After gathering violations, check per-file and per-line ignores and discard those we
  don't care about.
- Install executable
- Python package (see how ruff does it, use `maturin`).
- Publish to `crates.io` and PyPI.
