# assumed-size (T041)
## What does it do?
Checks for assumed size variables

## Why is this bad?
Assumed size dummy arguments declared with a star `*` as the size should be
avoided. There are several downsides to assumed size, the main one being
that the compiler is not able to determine the array bounds, so it is not
possible to check for array overruns or to use the array in whole-array
expressions.

Instead, prefer assumed shape arguments, as the compiler is able to keep track of
the upper bounds automatically, and pass this information under the hood. It also
allows use of whole-array expressions, such as `a = b + c`, where `a, b, c` are
all arrays of the same shape.

Instead of:

```f90
subroutine process_array(array)
    integer, dimension(*), intent(in) :: array
    ...
```

use:

```f90
subroutine process_array(array)
    integer, dimension(:), intent(in) :: array
    ...
```

Note that this doesn't apply to `character` types, where `character(len=*)` is
actually the most appropriate specification for `intent(in)` arguments! This is
because `character(len=:)` must be either a `pointer` or `allocatable`.