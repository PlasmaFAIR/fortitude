# too-complex (S901)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for procedures with a cyclomatic complexity that exceeds a
configurable threshold.

## Why is this bad?
Cyclomatic complexity measures the number of linearly independent paths
through a procedure. A high complexity indicates that a procedure has too
many branches, making it harder to read, test, and maintain. Procedures
with a complexity above 10 (the threshold proposed by McCabe) are
generally considered too complex and should be refactored.

## Example
```f90
subroutine classify(x, category)
  real, intent(in) :: x
  integer, intent(out) :: category
  if (x < 0.0) then
    if (x < -100.0) then
      category = 1
    else if (x < -10.0) then
      category = 2
    else
      category = 3
    end if
  else if (x == 0.0) then
    category = 4
  else
    if (x > 100.0) then
      category = 5
    else if (x > 10.0) then
      category = 6
    else
      category = 7
    end if
  end if
end subroutine classify
```

Use instead:
```f90
integer function classify_negative(x)
  real, intent(in) :: x
  if (x < -100.0) then
    classify_negative = 1
  else if (x < -10.0) then
    classify_negative = 2
  else
    classify_negative = 3
  end if
end function classify_negative

integer function classify_positive(x)
  real, intent(in) :: x
  if (x > 100.0) then
    classify_positive = 5
  else if (x > 10.0) then
    classify_positive = 6
  else
    classify_positive = 7
  end if
end function classify_positive

subroutine classify(x, category)
  real, intent(in) :: x
  integer, intent(out) :: category
  if (x < 0.0) then
    category = classify_negative(x)
  else if (x == 0.0) then
    category = 4
  else
    category = classify_positive(x)
  end if
end subroutine classify
```

## Options
- [`check.too-complex.max-complexity`][check.too-complex.max-complexity]

## References
- [Wikipedia: Cyclomatic complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity)

[check.too-complex.max-complexity]: ../settings.md#check_too-complex_max-complexity

