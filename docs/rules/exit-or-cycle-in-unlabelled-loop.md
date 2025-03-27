# exit-or-cycle-in-unlabelled-loop (C142)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for `exit` or `cycle` in unnamed `do` loops

## Why is this bad?
Using loop labels with `exit` and `cycle` statements prevents bugs when exiting the
wrong loop, and helps readability in deeply nested or long loops. The danger is
particularly enhanced when code is refactored to add further loops.

## Settings
See [allow-unnested-loops](../settings.md#check_exit-unlabelled-loops_allow-unnested-loops)
