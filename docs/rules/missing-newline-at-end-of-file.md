# missing-newline-at-end-of-file (S002)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Checks for the absence of a new line at the end of the file.

## Why is this bad?
POSIX standards state that a line is a sequence of characters
ending with a newline character. Some tools may not handle files
that do not end with a newline correctly, leading to potential issues
in file processing, version control diffs, and concatenation of files.
