# redirected-allow-comment (FORT003)
Fix is always available.

This rule is turned on by default.

## What it does
Checks for redirected rules in allow comments.

## Why is this bad?
When one of Fortitude's rule codes has been redirected, the implication is that the rule has
been deprecated in favor of another rule or code. To keep your codebase
consistent and up-to-date, prefer the canonical rule code over the deprecated
code.

## Example
```f90
! allow(T001)
program foo
```

Use instead:
```f90
! allow(implicit-typing)
program foo
```