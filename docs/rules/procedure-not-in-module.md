# procedure-not-in-module (M001)
This rule is turned on by default.

## What it does
Checks for any functions and subroutines not defined within modules (or one
of a few acceptable alternatives).

## Why is this bad?
Functions and subroutines should be contained within (sub)modules or programs.
Fortran compilers are unable to perform type checks and conversions on functions
defined outside of these scopes, and this is a common source of bugs.