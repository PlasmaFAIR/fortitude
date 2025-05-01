# if-statement-semicolon (C151)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Checks for misleading semicolons in `if` statements.

## Why is this bad?
The following code may appear to execute two statements only if the `if`
condition is true, but in actuality the second statement will always be
executed:

```f90
if (condition) print *, "Hello"; print *, "World"
```

It is equivalent to:

```f90
if (condition) then
   print *, "Hello"
end if
print *, "World"
```

When applying fixes, the if statement is converted to the second form and
the semicolon is removed.
