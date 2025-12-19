# missing-action-specifier (C043)
## What it does
Checks for missing action specifier when opening files.

## Why is this bad?
By default, files are opened in `readwrite` mode, but this may not be the
programmer's intent. Explicitly specifying `read`, `write` or `readwrite`
makes it clear how the file is intended to be used, and prevents the
accidental overwriting of input data.
