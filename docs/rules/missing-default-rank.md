# missing-default-rank (C013)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that `select rank` statements have a `rank default`.

## Why is this bad?
Select statements without a default can lead to incomplete handling of
the possible options. If the given rank isn't handled by any of the cases, the
program will continue execution, which may lead to surprising results. This
is a common source of bugs if the processing is rank-specific, and especially
if the variable is part of the arguments to a function/subroutine. Having a
default allows for the program to gracefully handle errors.

## Examples

Instead of:

```f90
select rank(A)
rank (0)
    ! Scalar
    call scalarVersion(A)
rank (1)
    call vectorVersion(A)
end select
```

use:

```f90
select rank(A)
rank (0)
    ! Scalar
    call scalarVersion(A)
rank (1)
    call vectorVersion(A)
rank default
    call handle_error("Unsupported rank: ", rank(A))
end select
```

If you do only intend to handle a subset of ranks, you can use a `continue`
statement with an explanatory comment:

```f90
select rank(A)
rank (0)
    ! Scalar
    call scalarVersion(A)
rank (1)
    call vectorVersion(A)
rank default
    ! Other ranks handled elsewhere
    continue
end select
```

You may also consider instead using an `if` statement. This can make your
intention more obvious.
