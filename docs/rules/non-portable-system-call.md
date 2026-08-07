# non-portable-system-call (PORT061)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of the non-portable `system` call for running programs.

## Why is this bad?
`system` is a GFortran extension and isn't available as part of other compilers.

## Example
```f90
call system("dir")
```

Use instead:
```f90
call execute_command_line("dir")
```
