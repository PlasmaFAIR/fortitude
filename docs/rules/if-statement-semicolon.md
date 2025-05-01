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
if (condition) print *, "Hello"
print *, "World"
```

Users should be cautious applying this fix. If the intent was to have
both statements execute only if the condition is true, then the user
should rewrite the code to use an `if` statement with a block:

```f90
if (condition) then
    print *, "Hello"
    print *, "World"
end if
```
