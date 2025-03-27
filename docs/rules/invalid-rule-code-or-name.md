# invalid-rule-code-or-name (FORT001)
This rule is turned on by default.

## What it does
Checks for invalid rules in allow comments.

## Why is this bad?
Invalid rules in allow comments are likely typos or mistakes.

## Example
The user meant `implicit-typing` but made a mistake:
```f90
! allow(implicit-typos)
program test
end program test
```
