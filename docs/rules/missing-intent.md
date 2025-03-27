# missing-intent (C061)
This rule is turned on by default.

## What it does
Checks for missing `intent` on dummy arguments

## Why is this bad?
Procedure dummy arguments should have an explicit `intent`
attributes. This can help catch logic errors, potentially improve
performance, as well as serving as documentation for users of
the procedure.

Arguments with `intent(in)` are read-only input variables, and cannot be
modified by the routine.

Arguments with `intent(out)` are output variables, and their value on
entry into the routine can be safely ignored.

Finally, `intent(inout)` arguments can be both read and modified by the
routine. If an `intent` is not specified, it will default to
`intent(inout)`.
