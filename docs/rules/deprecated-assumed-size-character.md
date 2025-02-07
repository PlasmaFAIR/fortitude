# deprecated-assumed-size-character (B062)
## What does it do?
Checks for deprecated declarations of `character`

## Why is this bad?
The syntax `character*(*)` is a deprecated form of `character(len=*)`. Prefer the
second form.