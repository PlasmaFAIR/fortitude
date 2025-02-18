# initialisation-in-declaration (T051)
This rule is turned on by default.

## What it does
Checks for local variables with implicit `save`

## Why is this bad?
Initialising procedure local variables in their declaration gives them an
implicit `save` attribute: the initialisation is only done on the first call
to the procedure, and the variable retains its value on exit.

## Examples
For example, this subroutine:

```f90
subroutine example()
  integer :: var = 1
  print*, var
  var = var + 1
end subroutine example
```

when called twice:

```f90
call example()
call example()
```

prints `1 2`, when it might be expected to print `1 1`.

Adding the `save` attribute makes it clear that this is the intention:

```f90
subroutine example()
  integer, save :: var = 1
  print*, var
  var = var + 1
end subroutine example
```

Unfortunately, in Fortran there is no way to disable this behaviour, and so if it
is not intended, it's necessary to have a separate assignment statement:

```f90
subroutine example()
  integer :: var
  var = 1
  print*, var
  var = var + 1
end subroutine example
```

If the variable's value is intended to be constant, then use the `parameter`
attribute instead:

```f90
subroutine example()
  integer, parameter :: var = 1
  print*, var
end subroutine example
```