# exit-or-cycle-in-unlabelled-loop (C142)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for `exit` or `cycle` in unnamed `do` loops

## Why is this bad?
Using loop labels with `exit` and `cycle` statements prevents bugs from
exiting the wrong loop. The danger is particularly enhanced when code is
refactored to add further loops.

## Settings
See [nested-loops-only](../settings.md#check_exit-labelled-loops_nested-loops-only)