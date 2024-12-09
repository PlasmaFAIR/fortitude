# Fortitude: A Fortran Linter

A Fortran linter, inspired by (and built upon) [Ruff](https://github.com/astral-sh/ruff).
Written in Rust :crab: and installable with Python :snake:.

- :zap: Blazingly fast, up to hundreds of times faster than other open-source Fortran
  linters.
- :wrench: Automatically fixes linter warnings.
- :chart_with_upwards_trend: 30+ rules, with many more planned.
- :page_with_curl: Multiple output formats, including SARIF and GitHub/GitLab CI.
- :handshake: Follows [community best
  practices](https://fortran-lang.org/en/learn/best_practices/).
- :muscle: Built on a robust [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
  parser -- even syntax errors won't stop it!

## Quickstart

Fortitude can be installed directly into your Python environment:

```console
$ pip install fortitude-lint
```

Run Fortitude on your project:

```console
$ fortitude check
```
