# unchecked-allocate-stat (C181)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
If the status of an `allocate` statement is checked by passing a variable to
the `stat` argument, that variable must be checked. To avoid confusing and
bug-prone control flow, the `stat` variable should be checked within the
same scope as the `allocate` statement.

## Why is this bad?
By default, `allocate` statements will crash the program if the allocation
fails. This is often the desired behaviour, but to provide for cases in
which the user wants to handle allocation errors gracefully, they may
optionally check the status of an `allocate` statement by passing a variable
to the `stat` argument:

```f90
allocate (x(BIG_INT), stat=status)
if (status /= 0) then
  call handle_error(status)
end if
```

However, if the `stat` variable is not checked, the program will continue to
run despite the allocation failure, which can lead to undefined behaviour.
