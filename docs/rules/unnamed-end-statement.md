# unnamed-end-statement (S061)
Fix is always available.

This rule is turned on by default.

## What does it do?
Checks that `end` statements include the type of construct they're ending

## Why is this bad?
End statements should specify what kind of construct they're ending, and the
name of that construct. For example, prefer this:

```f90
module mymodule
  ...
end module mymodule
```

To this:

```f90
module mymodule
  ...
end
```

Or this:

```f90
module mymodule
  ...
end module
```

Similar rules apply for many other Fortran statements
