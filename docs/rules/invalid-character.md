# invalid-character (PORT032)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for the use of invalid characters in source code (except strings and
comments)

## Why is this bad?
The Fortran standard only supports the basic ASCII character set (`a-z, A-Z,
0-9`, and some punctuation), and the vast majority of compilers will error
on non-ASCII characters, for example letters with diacritics or accents.
