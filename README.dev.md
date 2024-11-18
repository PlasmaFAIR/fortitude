# Fortitude Development

## Installation from source

To install from source, you must first have a working Rust environment (see
[rustup](https://rustup.rs/)). The project may then be installed from the project
root directory using either `pip`:

```bash
python -m venv venv # Or use your preferred virtual environment method...
source venv/bin/activate
pip install .[lint]
```

Or using `cargo`:

```bash
cargo install --path fortitude
```

## Testing

Unit tests can be run by calling:

```bash
cargo test
```

You'll also need [Insta](https://insta.rs/docs/) to update snapshot tests:

```shell
cargo install cargo-insta
```

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

#### Rule testing: fixtures and snapshots

To test rules, Fortitude uses snapshots of Fortitude's output for a given file (fixture). Generally, there
will be one file per rule (e.g., `E402.py`), and each file will contain all necessary examples of
both violations and non-violations. `cargo insta review` will generate a snapshot file containing
Fortitude's output for each fixture, which you can then commit alongside your changes.

Once you've completed the code for the rule itself, you can define tests with the following steps:

1. Add a Python file to `fortitude/resources/test/fixtures/[category]` that contains the code you
    want to test. The file name should match the rule name (e.g., `E402.py`), and it should include
    examples of both violations and non-violations.

1. Run Fortitude locally against your file and verify the output is as expected. Once you're satisfied
    with the output (you see the violations you expect, and no others), proceed to the next step.
    For example, if you're adding a new rule named `T402`, you would run:

    ```shell
    cargo run -- check fortitude/resources/test/fixtures/typing/T402.f90 --select T402
    ```

    **Note:** Only a subset of rules are enabled by default. When testing a new rule, ensure that
    you activate it by adding `--select ${rule_code}` to the command.

1. Add the test to the relevant `fortitude/src/rules/[category]/mod.rs` file. If you're contributing
    a rule to a pre-existing set, you should be able to find a similar example to pattern-match
    against. If you're adding a new category, you'll need to create a new `mod.rs` file (see,
    e.g., `fortitude/src/rules/typing/mod.rs`)

1. Run `cargo test`. Your test will fail, but you'll be prompted to follow-up
    with `cargo insta review`. Run `cargo insta review`, review and accept the generated snapshot,
    then commit the snapshot file alongside the rest of your changes.

1. Run `cargo test` again to ensure that your test passes.


## Making New Releases

To make a new release, the following steps must be completed in order:

- Make a new commit that updates the project version in `pyproject.toml`,
  `Cargo.toml`, and `CITATION.cff`.
  - Remember to run `cargo build` to update the `Cargo.lock` file too!
- Open a new PR to merge this change.
- After merging, make a new release on GitHub.
  - This will automatically upload the new version to PyPI.
- On your machine, pull the main branch, and run `cargo publish`.
  - This will upload the Rust crate to `crates.io`.
