# unused-allow-comment (FORT002)
Fix is always available.

This rule is turned on by default.

## What it does
Checks for `allow` comments that aren't applicable.

## Why is this bad?
An `allow` comment that no longer matches any diagnostic violations
is likely included by mistake, and should be removed to avoid confusion.

## Example
```f90
! allow(implicit-typing)
program foo
  implicit none
```

Use instead:
```f90
program foo
  implicit none
```
