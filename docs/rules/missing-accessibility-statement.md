# missing-accessibility-statement (M021)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for missing `private` or `public` accessibility statements in modules

## Why is this bad?
The `private` statement makes all entities (variables, types, procedures)
private by default, requiring an explicit `public` attribute to make them
available. As well as improving encapsulation between modules, this also
makes it possible to detect unused entities.

A `public` statement in a module does not change the default behaviour,
and therefore all entities will be available from outside the module
unless they are individually given a `private` attribute. This brings
all of the same downsides as the default behaviour, but an explicit
`public` statement makes it clear that the programmer is choosing
this behaviour intentionally.