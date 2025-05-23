# keyword-has-whitespace (S232)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for the use of `in out` instead of `inout` and `go to` instead of `goto`.
Either may be exempted from this rule by setting the options
[`inout-with-space`](../settings.md#inout-with-space) and
[`goto-with-space`](../settings.md#goto-with-space).

## Why is this bad?
By convention, `inout` in normally preferred to `in out`. Both `go to` and
`goto` are valid, but Fortitude prefers the latter as `goto` is most common
in other languages, and neither `go` nor `to` have secondary purposes in
other keywords. Enforcing this rule can help maintain a consistent style.
