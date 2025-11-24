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

Arguments with `intent(out)` are output variables, and their value
on entry into the routine can be safely ignored -- technically,
they become undefined on entry, which includes deallocation and/or
finalisation.

Finally, `intent(inout)` arguments can be both read and modified
by the routine. If an `intent` is not specified, it will default
to essentially `intent(inout)` -- however, this can be dangerous
if passing literals or expressions that can't be modified.

This rule will permit the absence of `intent` for dummy arguments
that include the `value` attribute. It will also permit `pointer`
dummy arguments that lack an `intent` attribute in standards prior
to Fortran 2003, in which `pointer` dummy arguments were not
allowed to have `intent`.
