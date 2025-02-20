# implicit-typing (C051)
This rule is turned on by default.

## What does it do?
Checks for missing `implicit none`

## Why is this bad?
'implicit none' should be used in all modules and programs, as implicit typing
reduces the readability of code and increases the chances of typing errors.