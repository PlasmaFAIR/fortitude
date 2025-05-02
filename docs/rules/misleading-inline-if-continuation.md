# misleading-inline-if-continuation (C152)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Checks for misleading line continuations in inline `if` statements.

## Why is this bad?
An inline `if` statement followed immediately by a line continuation
can be easily confused for a block `if` statement:

```f90
if (condition) &
    call a_very_long_subroutine_name(with, many, ..., arguments)
```

If a developer wishes to add a second statement to the `if` 'block',
they may be tempted to write:

```f90
if (condition) &
    call a_very_long_subroutine_name(with, many, ..., arguments)
    call another_subroutine(args, ...)  ! Always executes!
```

To avoid this confusion, inline `if` statements that spill over multiple
lines should be written as an if-then-block:

```f90
if (condition) then
    call a_very_long_subroutine_name(with, many, ..., arguments)
    call another_subroutine(args, ...)  ! Only executes if condition is true
end if
```
