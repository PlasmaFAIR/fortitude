# missing-default-type (C012)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that `select type` statements have a `class default`.

## Why is this bad?
Select statements without a default can lead to incomplete handling of
the possible options. If the type isn't handled by any of the classes, the
program will continue execution, which may lead to surprising results.  This
is a common source of bugs when adding new types or options, as it's easy to forget
to update all `select` statements. Having a default allows for the program to
gracefully handle errors.

## Examples

Instead of:

```f90
select type(pet)
type is (dog_t)
    call routine1()
class is (animal_t)
    call routine2()
end select
```

use:

```f90
select type(pet)
type is (dog_t)
    call routine1()
class is (animal_t)
    call routine2()
class default
   call handle_error("Invalid pet: ", pet)
end select
```

If you do only intend to handle a subset of types, you can use a `continue`
statement with an explanatory comment:

```f90
select type(pet)
type is (dog_t)
    call routine1()
class is (animal_t)
    call routine2()
class default
    ! Other pet types handled elsewhere
    continue
end select
```

You may also consider instead using an `if` statement. This can make your
intention more obvious.
