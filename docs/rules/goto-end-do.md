# goto-end-do (OB094)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `goto` statements that target `end do` statements.

## Why is this bad?
`goto` is generally considered harmful because it encourages unstructured
programming, making it much harder to understand the control flow of the
code. `goto` statements that point to the end of loops can be replaced with
`cycle` statements instead. These make the programmer's intentions much
clearer.

When a `goto` is used like this in a nested loop, the loops should instead
use named constructs (see
[`exit-or-cycle-in-unlabelled-loop`](exit-or-cycle-in-unlabelled-loop.md).

## Example
```f90
    do 20 i = 1, 10
        if (i > 5) goto 20
        foo(i) = 2 * i
20  end do
```

Use instead:
```f90
    do i = 1, 10
        if (i > 5) cycle
        foo(i) = 2 * i
    end do
```
