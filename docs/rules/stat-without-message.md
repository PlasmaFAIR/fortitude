# stat-without-message (C183)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
This rule detects whether `stat` is used without also setting `errmsg` when
allocating or deallocating. Similarly checks for the use of `iostat` without
`iomsg` with IO routines, and `cmdstat` without `cmdmsg` when using
`execute_command_line`.

## Why is this bad?
The error codes returned when using `stat`, `iostat`, or `cmdstat` are not
very informative on their own, and are not portable across compilers. It is
recommended to always capture the associated error message alongside the
error code:

```f90
real, allocatable :: x(:)
integer :: status
character(256) :: message ! N.B. Can be allocatable in F2023+

allocate (x(100), stat=status, errmsg=message)
open (unit=10, file="data.txt", iostat=status, iomsg=message)
call execute_command_line("ls", cmdstat=status, cmdmsg=message)
```
