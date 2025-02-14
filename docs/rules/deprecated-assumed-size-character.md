# deprecated-assumed-size-character (T043)
This rule is turned on by default.

## What does it do?
Checks for deprecated declarations of `character`

## Why is this bad?
The syntax `character*(*)` is a deprecated form of `character(len=*)`. Prefer the
second form.