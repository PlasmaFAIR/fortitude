# non-portable-intrinsic-procedure (PORT061)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of non-portable intrinsic procedures.

## Why is this bad?
Many compilers include extra intrinsic procedures which provide
functionality which was not in the Fortran standard at the time of the
compiler's release, although they often become standardised in later
versions. These extensions may not be available in other compilers, or may
work differently, making the code non-portable.

## Example
```f90
call system("dir")
```

Use instead:
```f90
call execute_command_line("dir")
```

## Extensions and their standard equivalents

| Extension          | Standard                       | Notes |
|--------------------|--------------------------------|-------|
| [`system`][system] | [`execute_command_line`][execute_command_line] | Extension can be either subroutine or function, standard version is a subroutine only |
| [`exit`][exit]     | `stop`                           | Extension is a subroutine, standard version is a statement |
| [`abort`][abort]   | `error stop`                     | Extension is a subroutine, standard version is a statement |
| [`srand`][srand]   | [`random_seed`][random_seed]     | Standard has different semantics |
| [`rand`][rand]     | [`random_number`][random_number] | Standard is a subroutine rather than function |

## Fix safety
### `system`
When the gfortran `system` extension is used as a subroutine, it has nearly
identical semantics to the standard `execute_command_line`, except that the
`status` argument should be replaced with the `exitstat` keyword argument
instead.

However, `system` can also be used as a function, in which case a fix may be
more complicated if used as part of a larger expression.

### `exit` and `abort`
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

### `srand` and `rand`
```f90
call srand(seed)
x = rand(seed)
```

#### Use instead
```f90
integer, allocatable :: seed(:)
integer :: n

call random_seed(size = n)
allocate(seed(n))
seed(1) = 12345
call random_seed(put=seed)
call random_number(x)
```

Prefer using the Fortran standard intrinsics `random_seed` and `random_number` instead.
These are not interchangeable replacements, as for example, `random_number(x)` is a subroutine
that modifies `x`, whereas `rand()` is a function that returns a single `real`.

## References
- [GNU Fortran intrinsic procedures](https://gcc.gnu.org/onlinedocs/gfortran/Intrinsic-Procedures.html)

[abort]: https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html
[execute_command_line]: https://gcc.gnu.org/onlinedocs/gfortran/EXECUTE_005fCOMMAND_005fLINE.html
[exit]: https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html
[rand]: https://gcc.gnu.org/onlinedocs/gfortran/RAND.html
[random_number]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html
[random_seed]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html
[srand]: https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html
[system]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM.html
