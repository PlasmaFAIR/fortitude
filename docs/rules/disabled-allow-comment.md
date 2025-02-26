# disabled-allow-comment (FORT005)
Fix is always available.

This rule is turned on by default.

## What it does
Checks for `allow` comments that are disabled globally.

## Why is this bad?
These `allow` comments will have no effect, and should be removed to avoid
confusion.