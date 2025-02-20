# superfluous-implicit-none (S201)
Fix is always available.

This rule is turned on by default.

## What it does
Checks for unnecessary `implicit none` in module procedures

## Why is this bad?
If a module has 'implicit none' set, it is not necessary to set it in contained
functions and subroutines (except when using interfaces).