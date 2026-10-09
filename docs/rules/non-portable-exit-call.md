# non-portable-exit-call (PORT063)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of the non-portable `exit` and `abort` subroutines.

## Why is this bad?
`exit` and `abort` are GNU extensions and aren't available in other
compilers. The standard `stop` and `error stop` statements should be used
instead.

## Example
```f90
call exit(1)
call abort
```

Use instead:
```f90
stop 1
error stop
```

## Fix safety
The fix is unsafe because the replacements are not exact equivalents:
`stop` prints a `STOP` message to stderr, whereas `call exit` is silent, and
`error stop` terminates normally with a non-zero exit code, whereas `call
abort` raises a signal and may produce a core dump. The fix also can't tell
whether `exit` or `abort` refers to an external procedure of the same name.

A fix is only offered when the target standard supports it and the call has
at most one positional argument. A string literal is always accepted as a
stop code. An integer literal is too, except that literals with more than 5
digits need Fortran 2008. Any other expression requires Fortran 2018, and
`error stop` requires Fortran 2008. Some compilers (such as Intel and NAG)
accept an argument to `abort`, whereas gfortran does not.

## References
- [GFortran docs for `exit`](https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html)
- [GFortran docs for `abort`](https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html)
