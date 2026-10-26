# empty-branch (C153)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for `if`, `else if` and `else` branches that contain no statements.
Branches that only contain comments are also reported.

## Why is this bad?
An empty branch is usually left over from code that was removed or commented
out, or from something that was forgotten. If the branch is meant to do
nothing, say so explicitly with a `continue` statement.

## Example
```f90
if (flag) then
    call do_something()
else
end if
```

Use instead:
```f90
if (flag) then
    call do_something()
end if
```

Or, if the empty branch is intentional:
```f90
if (flag) then
    call do_something()
else
    continue
end if
```
