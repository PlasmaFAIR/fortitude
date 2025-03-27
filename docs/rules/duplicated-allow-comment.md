# duplicated-allow-comment (FORT004)
Fix is always available.

This rule is turned on by default.

## What it does
Checks for `allow` comments with duplicated rules.

## Why is this bad?
Duplicated rules in `allow` comments are very likely to be mistakes, and
should be removed to avoid confusion.

## Example
```f90
! allow(C001, C002, C001)
program foo
```

Use instead:
```f90
! allow(C001, C002)
program foo
```
