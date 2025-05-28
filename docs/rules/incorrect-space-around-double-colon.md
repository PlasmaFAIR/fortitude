# incorrect-space-around-double-colon (S103)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for `::` that aren't surrounded by a space on either side.

## Why is this bad?
Omitting any whitespace surrounding the double colon separator can make code harder
to read:

```f90
character(len=256)::x
! vs
character(len=256) :: x
```
