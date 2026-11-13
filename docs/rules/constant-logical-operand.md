# constant-logical-operand (C154)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for a constant `.true.` or `.false.` as an operand of `.and.` or `.or.`.

## Why is this bad?
A constant operand either cannot change the result or fixes it, so the
expression does not test what it appears to test. Both shapes usually come
from an edit that left a placeholder behind or from a condition that was
rewritten by halves.

## Example
```f90
if (is_ready .or. .true.) then
    call run()
end if
```
Use instead:
```f90
if (is_ready) then
    call run()
end if
```
