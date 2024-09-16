# Fortitude Development

## Installation from source

To install from source, you must first have a working Rust environment (see
[rustup](https://rustup.rs/)). The project may then be installed either using `pip`:

```bash
python -m venv venv # Or use your preferred virtual environment method...
source venv/bin/activate
pip install .[lint]
```

Or using `cargo`:

```bash
cargo install --path .
```

## Testing

Unit tests can be run by calling:

```bash
cargo test
```

Integration testing is currently being performed manually using the file `test.f90`:

```bash
fortitude check test.f90
```

The test suite is still in need of work, and we hope to include automated integration
tests soon.

## Linting and Formatting

When contributing, please use `cargo clippy` for linting and `cargo fmt` for formatting.
If you edit any Python code, please also use `ruff check` and `ruff format`. To avoid
accidentally pushing unlinted/unformatted code to GitHub, we recommend using the Git
pre-commit hook provided:

```bash
git config --local core.hooksPath .githooks
```

## Adding Rules

Similarly to how [Ruff](https://docs.astral.sh/ruff/) names rules, the rules in
Fortitude should be categorised appropriately and their name should describe the pattern
the rule is intended to fix. Words such as 'forbid' should be omitted. For example, the
name for the rule that warns of overly long lines is `LineTooLong`, and not something
like `AvoidLineTooLong` or `KeepLinesShort`.

## Making New Releases

To make a new release, the following steps must be completed in order:

- Make a new commit that updates the project version in `pyproject.toml` and
  `Cargo.toml`.
  - Remember to run `cargo build` to update the `Cargo.lock` file too!
- Open a new PR to merge this change.
- After merging, make a new release on GitHub.
  - This will automatically upload the new version to PyPI.
- On your machine, pull the main branch, and run `cargo publish`.
  - This will upload the Rust crate to `crates.io`.
