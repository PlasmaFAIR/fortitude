# single-quote-string (S241)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Catches use of single-quoted strings.

## Why is this bad?
For consistency, all strings should be either single-quoted or double-quoted.
Here, we enforce the use of double-quoted strings as this is the most common
style in other languages.

An exception is made for single-quoted strings that contain a `'"'` character,
as this is the preferred way to include double quotes in a string.

Fixes are not available for single-quoted strings containing escaped single
quotes (`"''"`).
