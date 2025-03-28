# keywords-missing-space (S231)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for the use of keywords comprised of two words where the space is
omitted, such as `elseif` instead of `else if` and `endmodule` instead of
`endmodule`. The keywords `inout` and `goto` are exempt from this rule by
default, but may be included by supplying the relevant options

TODO list options

## Why is this bad?
Contracting two keywords into one can make code less readable
