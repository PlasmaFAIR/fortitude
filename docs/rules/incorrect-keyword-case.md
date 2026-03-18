# incorrect-keyword-case (S233)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that Fortran keywords use a consistent casing style. Flags any keyword
whose casing does not match the configured style (see
[`keyword-case`](../settings.md#keyword-case)).

## Why is this bad?
Fortran is case-insensitive, so keyword casing is purely a stylistic choice.
However, inconsistent casing — mixing `IMPLICIT NONE`, `implicit none`, and
`Implicit None` in the same codebase — reduces readability and makes code
harder to scan. Enforcing a consistent style helps maintain a uniform
appearance across the codebase.

Modern Fortran style guides generally favour lowercase keywords. Older
codebases often use uppercase, inherited from fixed-form Fortran conventions.

## Examples

With `keyword-case = "lowercase"`:

### Incorrect
```fortran
IMPLICIT NONE
INTEGER, INTENT(IN) :: x
END SUBROUTINE foo
```

### Correct
```fortran
implicit none
integer, intent(in) :: x
end subroutine foo
```

With `keyword-case = "uppercase"`:

### Incorrect
```fortran
implicit none
integer, intent(in) :: x
end subroutine foo
```

### Correct
```fortran
IMPLICIT NONE
INTEGER, INTENT(IN) :: x
END SUBROUTINE foo
```
