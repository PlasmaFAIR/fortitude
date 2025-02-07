# interface-implicit-typing (B022)
## What it does
Checks for missing `implicit none` in interfaces

## Why is this bad?
Interface functions and subroutines require 'implicit none', even if they are
inside a module that uses 'implicit none'.