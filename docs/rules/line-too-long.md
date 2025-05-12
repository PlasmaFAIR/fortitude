# line-too-long (S001)
This rule is turned on by default.

## What does it do?
Checks line length isn't too long

## Why is this bad?
Long lines are more difficult to read, and may not fit on some developers'
terminals. The line continuation character '&' may be used to split a long line
across multiple lines, and overly long expressions may be broken down into
multiple parts.

The maximum line length can be changed using the flag `--line-length=N`. The
default maximum line length is 100 characters. This is a fair bit more than the
traditional 80, but due to the verbosity of modern Fortran it can sometimes be
difficult to squeeze lines into that width, especially when using large indents
and multiple levels of indentation.

Some lines that are longer than the maximum length may be acceptable, such as
long strings or comments. This is to allow for long URLs or other text that cannot
be reasonably split across multiple lines.

Note that the Fortran standard states a maximum line length of 132 characters,
and while some modern compilers will support longer lines, for portability it
is recommended to stay beneath this limit.
