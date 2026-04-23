# implicit-zero-digit (S281)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for floating point literals that begin or end in a bare decimal point,
such as `.5` or `2.`.

## Why is this bad?
Floating point literals that begin or end in a bare decimal point can be
difficult to read and may lead to confusion. For example, `.5` could be
misread as `5.`. It is generally recommended to include a leading zero
before the decimal point and a trailing zero after the decimal point for
clarity, such as `0.5` and `2.0`.
