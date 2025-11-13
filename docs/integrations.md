# Integrations

## pre-commit

Fortitude can be used as a [pre-commit](https://pre-commit.com/) hook via
[fortitude-pre-commit](https://github.com/PlasmaFAIR/fortitude-pre-commit):

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.7.5
  hooks:
    - id: fortitude
```

To enable fixes, add the [`--fix`](settings.md#fix) argument:

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.7.5
  hooks:
    - id: fortitude
      args: ["--fix"]
```

Other additional arguments are also accepted:

```yaml
repos:
- repo: https://github.com/PlasmaFAIR/fortitude-pre-commit
  # Fortitude version.
  rev: v0.7.5
  hooks:
    - id: fortitude
      args: ["--fix", "--preview", "--unsafe-fixes"]
```

When running with `--fix`, Fortitude's lint hook should be placed before any code
formatters in use.

Note that the Fortitude pre-commit hook will run with the option
[`--force-exclude`](settings.md#force-exclude) switched on by default.  This
will prevent Fortitude from checking files that have been explicitly added to
your exclude list in `fpm.toml`, `fortitude.toml` or `.fortitude.toml`.
