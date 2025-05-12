# split-escaped-quote (C171)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for Fortran-escaped quotes in string literals that have been split over two lines.

## Why is this bad?
In Fortran string literals, literal (escaped) double or single quotes are denoted
with two characters: `""` or `''`. A surprising Fortran feature is the ability to
split tokens over multiple lines, including these escaped quotes. The result is that
it's possible to mistake such a split escaped quote with implicit concatenation of
string literals, a feature in other languages but not in Fortran. Splitting escaped
quotes is practically never desired, and can be safely replaced with a simple line
continuation.

## Example
```f90
print*, "this looks like implicit "&
     &" concatenation but isn't"
end
```

Use instead:
```f90
print*, "this looks like implicit&
     & concatenation but isn't"
end
```
