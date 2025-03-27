# default-public-accessibility (C132)
## What it does
Checks if the default accessibility in modules is set to `public`

## Why is this bad?
The `public` statement makes all entities (variables, types, procedures)
public by default. This decreases encapsulation and makes it more likely to
accidentally expose more than necessary. Public accessibility also makes
it harder to detect unused entities, which can often be indicative of
errors within the code.
