# return-in-program (PORT041)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of `return` statement inside a `program` body, as allowed by some compilers.
Suggests to replace it with `stop`.

## Why is this bad?
It is non-standard and not portable.

## Example
```f90
program test
  return
end program test
```

Use instead:
```f90
program test
  stop
end program test
```
