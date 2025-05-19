# keywords-missing-space (S231)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for the use of keywords comprised of two words where the space is
omitted, such as `elseif` instead of `else if` and `endmodule` instead of
`endmodule`. The keywords `inout` and `goto` are exempt from this rule by
default, but may be included by setting the options
[`inout-with-space`](../settings.md#inout-with-space) and
[`goto-with-space`](../settings.md#goto-with-space).

## Why is this bad?
Contracting two keywords into one can make code less readable. Enforcing
this rule can help maintain a consistent style.
