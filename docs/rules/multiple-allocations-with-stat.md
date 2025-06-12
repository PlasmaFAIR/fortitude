# multiple-allocations-with-stat (C182)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
This rule detects whether `stat` is used alongside multiple allocations or
deallocations.

## Why is this bad?
When allocating or deallocating multiple variables at once, the use of a `stat`
parameter will permit the program to continue running even if one of the
allocations or deallocations fails. However, it may not be clear which
allocation or deallocation caused the error.

To avoid confusion, it is recommended to use separate allocate or deallocate
statements for each variable and check the `stat` parameters individually.
