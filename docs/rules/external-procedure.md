# external-procedure (T061)
This rule is turned on by default.

## What does it do?
Checks for procedures declared with just `external`

## Why is this bad?
Compilers are unable to check external procedures without an explicit
interface for errors such as wrong number or type of arguments.

If the procedure is in your project, put it in a module (see
`external-function`), or write an explicit interface.