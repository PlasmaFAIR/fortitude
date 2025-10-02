# Fortitude: A Fortran Linter

A Fortran linter, inspired by (and built upon) [Ruff](https://github.com/astral-sh/ruff).
Written in Rust :crab: and installable with Python :snake:.

<figure markdown="span">
  ![Bar chart with benchmark results](/assets/performance_plot_dark.svg#only-dark)
  ![Bar chart with benchmark results](/assets/performance_plot_light.svg#only-light)
  <figcaption>Linting 43 files from the GS2 repo</figcaption>
</figure>

- :zap: Blazingly fast, up to hundreds of times faster than other open-source Fortran
  linters.
- :wrench: Automatically fixes linter warnings.
- :chart_with_upwards_trend: 50+ rules, with many more planned.
- :page_with_curl: Multiple output formats, including SARIF and GitHub/GitLab CI.
- :handshake: Follows [community best
  practices](https://fortran-lang.org/learn/best_practices/).
- :muscle: Built on a robust [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
  parser.
