# incorrect-keyword-case (S233)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that Fortran keywords use a consistent casing style.

## Why is this bad?
Fortran is case-insensitive, so keyword casing is purely a stylistic choice.
However, inconsistent casing — mixing `IMPLICIT NONE`, `implicit none`, and
`Implicit None` in the same codebase — reduces readability and makes code
harder to scan. Enforcing a consistent style helps maintain a uniform
appearance across the codebase.

Modern Fortran style guides generally favour lowercase keywords, while older
codebases often use uppercase, inherited from fixed-form Fortran
conventions; the used case can be configured with
[`check.incorrect-keyword-case.keyword-case`][check.incorrect-keyword-case.keyword-case]

## Examples

With `keyword-case = "lowercase"`:

### Incorrect
```f90
IMPLICIT NONE
INTEGER, INTENT(IN) :: x
END SUBROUTINE foo
```

### Correct
```f90
implicit none
integer, intent(in) :: x
end subroutine foo
```

With `keyword-case = "uppercase"`:

### Incorrect
```f90
implicit none
integer, intent(in) :: x
end subroutine foo
```

### Correct
```f90
IMPLICIT NONE
INTEGER, INTENT(IN) :: x
END SUBROUTINE foo
```

## Options
- [`check.incorrect-keyword-case.keyword-case`][check.incorrect-keyword-case.keyword-case]


[check.incorrect-keyword-case.keyword-case]: ../settings.md#check_incorrect-keyword-case_keyword-case

