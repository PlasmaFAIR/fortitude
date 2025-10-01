# implicit-typing (C001)
This rule is turned on by default.

## What does it do?
Checks for missing `implicit none`.

## Why is this bad?
Very early Fortran determined the type of variables implicitly
from the first character of their name which saved lines in the
days of punchcards, and for backwards compatibility this is still
the default behaviour. However, the major downside is that typos
can silently introduce undefined variables and lead to hard to
track down bugs. For example:

```f90
do i = 1, 10
    print*, in
end do
```

will print garbage.

'implicit none' should be used in all modules and programs, as
implicit typing reduces the readability of code and increases the
chances of typing errors.
