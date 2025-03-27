# syntax-error (E001)
This rule is turned on by default.

## What it does
Checks for syntax errors

This rule reports any syntax errors reported by Fortitude's Fortran parser.
This may indicate an error with your code, an aspect of Fortran not recognised
by the parser, or a non-standard extension to Fortran that our parser can't
handle, such as a pre-processor.

If this rule is reporting valid Fortran, please let us know, as it's likely a
bug in our code or in our parser!
