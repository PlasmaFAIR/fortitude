# mutable-module-variable (C202)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for public mutable module variables

## Why is this bad?
Module variables that are accessible from outside their module can be modified
by any code that `use`s the module. This makes it hard to reason about their
value at any given point in a program, and can make testing and debugging
difficult: tests are no longer independent of each other, as state can leak
from one to the next.

Constants (those with the `parameter` attribute) are fine, as are variables
with the `protected` attribute, which cannot be modified outside the module
that declares them. Variables that are `private` are not flagged.

Instead of exposing a mutable variable, consider making it `private` and
providing procedures to get and set its value, or passing the value through
procedure arguments.

## Examples
```f90
module my_mod
  implicit none (type, external)
  private

  public :: counter  ! Bad: any module that uses `my_mod` can change this
  integer :: counter = 0
end module my_mod
```

Use instead:

```f90
module my_mod
  implicit none (type, external)
  private

  public :: get_counter, increment_counter
  integer :: counter = 0

contains
  integer function get_counter()
    get_counter = counter
  end function get_counter

  subroutine increment_counter()
    counter = counter + 1
  end subroutine increment_counter
end module my_mod
```

or, if the variable should be visible but not modifiable from outside:

```f90
module my_mod
  implicit none (type, external)
  private

  integer, public, protected :: counter = 0
end module my_mod
```
