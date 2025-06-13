# maths-missing-parentheses (S251)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks if mathematical expressions are missing parentheses when operators have
different precedences.

## Why is this bad?
Long or complex expressions can be difficult or confusing to read, especially when
mixing operators with different precedences. Adding parentheses can clarify the code
and make the author's intent clearer, reducing the likelihood of misunderstandings
or bugs.

## Example
```f90
x = 1. + 2. * 3. - 4. / 5.
```

Use instead:
```f90
x = 1. + (2. * 3.) - (4. / 5.)
```
