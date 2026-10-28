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

It is inherently tricky to identify compiler extensions, different compilers
may have intrinsic procedures with the same name but different behaviour for
example. So although we have automatic fixes for many of these, we recommend
that they are always reviewed by a human; hence we mark all the fixes as
[unsafe](../linter.md#fix-safety).

## Example
```f90
call system("dir")
```

Use instead:
```f90
call execute_command_line("dir")
```

## Extensions and their standard equivalents

Summary of compiler-specific intrinsics and their standard equivalents:

| Extension            | Standard                                           |
|----------------------|----------------------------------------------------|
| [`system`][system]   | [`execute_command_line`][execute_command_line]     |
| [`exit`][exit]       | `stop`                                             |
| [`abort`][abort]     | `error stop`                                       |
| [`srand`][srand]     | [`random_seed`][random_seed]                       |
| [`rand`][rand]       | [`random_number`][random_number]                   |
| [`iargc`][iargc]     | [`command_argument_count`][command_argument_count] |
| [`getarg`][getarg]   | [`get_command_argument`][get_command_argument]     |
| [`etime`][etime]     | Various, see below                                 |
| Time/date procedures | [`date_and_time`][date_and_time]                   |
| [`lnblnk`][lnblnk]   | [`len_trim`][len_trim]                             |

See below for details on the specific procedures:

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

### `iargc` and `getarg`
These are completely replaced with the standard `command_argument_count` and
`get_command_argument`, and are almost always a straight replacement,
although some compilers may support additional arguments to `getarg`.

### `etime`
Different compilers may support different semantics `etime`, including
calling as either a subroutine or function, and different meanings of the
return arguments. There is no straight replacement for `etime` in the
standard, but the following intrinsic procedures have similar functionality:

- [`cpu_time`][cpu_time]: CPU time elapsed in seconds
- [`date_and_time`][date_and_time]: Date and time in a standard format
- [`system_clock`][system_clock]: Possibly monotonic nanosecond-resolution clock, if available

MPI and OpenMP also have standard facilities for getting elapsed time, which
may be higher resolution.

### Time and date procedures
There are many different procedures for getting or formatting the time and
date in various ways, such as `itime`, `fdate`, `ctime`, and so on. They are
mostly all replaced with the standard [`date_and_time`][date_and_time].

## References
- [GNU Fortran intrinsic procedures](https://gcc.gnu.org/onlinedocs/gfortran/Intrinsic-Procedures.html)
- [Intel Fortran language reference](https://www.intel.com/content/www/us/en/docs/fortran-compiler/developer-guide-reference/2026-1/a-to-z-reference.html)

[abort]: https://gcc.gnu.org/onlinedocs/gfortran/ABORT.html
[command_argument_count]: https://gcc.gnu.org/onlinedocs/gfortran/COMMAND_005fARGUMENT_005fCOUNT.html
[cpu_time]: https://gcc.gnu.org/onlinedocs/gfortran/CPU_005fTIME.html
[date_and_time]: https://gcc.gnu.org/onlinedocs/gfortran/DATE_005fAND_005fTIME.html
[etime]: https://gcc.gnu.org/onlinedocs/gfortran/ETIME.html
[execute_command_line]: https://gcc.gnu.org/onlinedocs/gfortran/EXECUTE_005fCOMMAND_005fLINE.html
[exit]: https://gcc.gnu.org/onlinedocs/gfortran/EXIT.html
[get_command_argument]: https://gcc.gnu.org/onlinedocs/gfortran/GET_005fCOMMAND_005fARGUMENT.html
[getarg]: https://gcc.gnu.org/onlinedocs/gfortran/GETARG.html
[iargc]: https://gcc.gnu.org/onlinedocs/gfortran/IARGC.html
[lnblnk]: https://gcc.gnu.org/onlinedocs/gfortran/LNBLNK.html
[len_trim]: https://gcc.gnu.org/onlinedocs/gfortran/LEN_005fTRIM.html
[rand]: https://gcc.gnu.org/onlinedocs/gfortran/RAND.html
[random_number]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html
[random_seed]: https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html
[srand]: https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html
[system]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM.html
[system_clock]: https://gcc.gnu.org/onlinedocs/gfortran/SYSTEM_005fCLOCK.html
