# pointer-initialisation-in-declaration (C082)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for local pointer variables with implicit `save`

## Why is this bad?
Initialising procedure local pointer variables in their declaration gives them
an implicit `save` attribute: the initialisation is only done on the first call
to the procedure, and the pointer retains its associated status on exit.

However, this associated status makes no guarantee that the target of the pointer
is valid or stays allocated between procedure calls - potentially leading to cases
where future calls into a procedure will reference an unallocated variable.

## Examples
For example, this subroutine:

```f90
subroutine example()
  integer, target  :: bad
  integer, pointer :: var => null()

  if (.not.associated(var)) then
    bad = 1
    var => bad
  end if

  print *, var
end subroutine example
```

when called twice

```f90
call example()
call doAnotherBigThing()
call example()
```

will implicitly save the association of `var` with the variable `bad` in the first
procedure call, but that variable most likely won't be at the same location in memory in
the next call, so the value printed in the second procedure call might not be `1`.

Adding the `save` attribute makes it clear that this is the intention, however you should
also ensure that the target variable is also saved.

```f90
subroutine example()
  integer, target,  save :: bad
  integer, pointer, save :: var => null()

  if (.not.associated(var)) then
    bad = 1
    var => bad
  end if

  print *, var
end subroutine example
```

Unfortunately, in Fortran there is no way to disable this behaviour, and so if it
is not intended, it's necessary to have a separate nullification statement before use:

```f90
subroutine example()
  integer, target  :: bad
  integer, pointer :: var

  var => null()  ! or use nullify(var)

  if (.not.associated(var)) then
    bad = 1
    var => bad
  end if

  print *, var
end subroutine example
```

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Section 8.5.3/8.5.4.
- Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
  University Press, Rule 74, p. 99.
