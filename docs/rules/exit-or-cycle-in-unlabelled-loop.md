# exit-or-cycle-in-unlabelled-loop (C142)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for `exit` or `cycle` in unnamed `do` loops

## Why is this bad?
Using loop labels with `exit` and `cycle` statements prevents bugs when exiting the
wrong loop, and helps readability in deeply nested or long loops. The danger is
particularly enhanced when code is refactored to add further loops.

## Example
```f90
do i = 1, 10
  do j = 1, 10
    if (i + j > 5) cycle
  end do
end do
```

Use instead:
```f90
do i = 1, 10
  inner: do j = 1, 10
    if (i + j > 5) cycle inner
  end do inner
end do
```

## Settings
See [allow-unnested-loops](../settings.md#check_exit-unlabelled-loops_allow-unnested-loops)
