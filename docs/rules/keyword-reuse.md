# keyword-reuse (S311)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for the use of keywords when naming variables, modules, functions,
etc.

## Why is this bad?
The reuse of keywords as identifiers can be confusing to readers, and may cause
problems for some tools. Enforcing this rule can help maintain a consistent style.

## Examples

```f90
module program

  implicit none (type, external)
  private

contains

  subroutine function(stop)
    integer, intent(in) :: stop
    print *, stop
  end subroutine function

end module program
```
