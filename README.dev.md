# Fortitude Development



## Installation from source

To install from source, you must first have a working Rust environment (see
[rustup](https://rustup.rs/)). The project may then be installed either using `pip`:

```bash
pip install .
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

When contributing, please use `cargo clippy` to lint your code, and `cargo fmt` to
format it.
