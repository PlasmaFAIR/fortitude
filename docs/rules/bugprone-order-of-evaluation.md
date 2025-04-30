# bugprone-order-of-evaluation (C161)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of optional dummy arguments in the same logical expression as
`present(<arg>)`.

## Why is this bad?
Unlike many other languages, the Fortran standard doesn't mandate (or prohibit)
short-circuiting in logical expressions, so different compilers have different
behaviour when it comes to evaluating such expressions. This is commonly encountered
when using `present()` with an optional dummy argument and checking its value in the
same expression. Without short-circuiting, this can lead to segmentation faults when
the expression is evaluated if the argument isn't present.

Instead, you should nest the conditional statements, or use the Fortran 2023
"condtional expression" (also called ternary expressions in other
languages). Unfortunately, any `else` branches may need to be duplicated or
refactored to accommodate this change.

## Example
Don't do this:
```f90
integer function example(arg1)
  integer, optional, intent(in) :: arg1

  if (present(arg1) .and. arg1 > 2) then
    example = arg1 * arg1
  else
    example = 1
  end if
```

Use instead, noting that we either need to duplicate the `else` branch, or refactor
differently:
```f90
integer function example(arg1)
  integer, optional, intent(in) :: arg1

  if (present(arg1)) then
    if (arg1 > 2) then
      example = arg1 * arg1
    else
      example = 1
    end if
  else
    example = 1
  end if
```

Or with Fortran 2023 (not currently supported by most compilers!):
```f90
integer function example(arg1)
  integer, optional, intent(in) :: arg1

  example = present(arg1) ? (arg1 > 2 ? arg1 * arg1 : 1) : 1
```
Note that it's still not possible to use a compound logical expression here, so we
still duplicate the default value.

## References
- <https://www.scivision.dev/fortran-short-circuit-logic/>
