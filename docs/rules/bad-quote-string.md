# bad-quote-string (S241)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Catches use of single- or double-quoted strings, depending on the value of
[`check.strings.quotes`][check.strings.quotes] option.

## Why is this bad?
For consistency, all strings should be either single-quoted or double-quoted.
Exceptions are made for strings containing escaped quotes.

## Example
```f90
foo = 'bar'
```

Assuming `quotes` is set to `double`, use instead:
```f90
foo = "bar"
```

## Options
- [`check.strings.quotes`][check.strings.quotes]


[check.strings.quotes]: ../settings.md#check_strings_quotes

