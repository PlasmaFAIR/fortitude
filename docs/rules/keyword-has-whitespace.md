# keyword-has-whitespace (S232)
Fix is sometimes available.

This rule is turned on by default.

## What it does
Checks for the use of `in out` instead of `inout` and `go to` instead of `goto`.

## Why is this bad?
By convention, `inout` is normally preferred to `in out`. Both `go to` and
`goto` are valid, but Fortitude prefers the latter as `goto` is most common
in other languages, and neither `go` nor `to` have secondary purposes in
other keywords. Enforcing this rule can help maintain a consistent style.

Either keyword may be exempted from this rule by setting the options
[`check.keyword-whitespace.inout-with-space`][check.keyword-whitespace.inout-with-space] and
[`check.keyword-whitespace.goto-with-space`][check.keyword-whitespace.goto-with-space].

## Options
- [`check.keyword-whitespace.inout-with-space`][check.keyword-whitespace.inout-with-space]
- [`check.keyword-whitespace.goto-with-space`][check.keyword-whitespace.goto-with-space]


[check.keyword-whitespace.inout-with-space]: ../settings.md#check_keyword-whitespace_inout-with-space
[check.keyword-whitespace.goto-with-space]: ../settings.md#check_keyword-whitespace_goto-with-space

