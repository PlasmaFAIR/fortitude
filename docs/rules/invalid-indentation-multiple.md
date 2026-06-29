# invalid-indentation-multiple (S105)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that the correct indentation has been used

The complexity of handling semicolons requires that this
rule removes any semicolons used midway through a line

## Why is this bad?
Inconsistent indentation makes Fortran less readable and difficult to
understand the scoping of logic.

## Options
- [`check.indent-width`][check.indent-width]
- [`check.invalid-indentation-multiple.num-indents-for-associate-contents`][check.invalid-indentation-multiple.num-indents-for-associate-contents]
- [`check.invalid-indentation-multiple.num-indents-for-block-contents`][check.invalid-indentation-multiple.num-indents-for-block-contents]
- [`check.invalid-indentation-multiple.num-indents-for-derived-type-contents`][check.invalid-indentation-multiple.num-indents-for-derived-type-contents]
- [`check.invalid-indentation-multiple.num-indents-for-do-contents`][check.invalid-indentation-multiple.num-indents-for-do-contents]
- [`check.invalid-indentation-multiple.num-indents-for-function-contents`][check.invalid-indentation-multiple.num-indents-for-function-contents]
- [`check.invalid-indentation-multiple.num-indents-for-if-contents`][check.invalid-indentation-multiple.num-indents-for-if-contents]
- [`check.invalid-indentation-multiple.num-indents-for-interface-contents`][check.invalid-indentation-multiple.num-indents-for-interface-contents]
- [`check.invalid-indentation-multiple.num-indents-for-module-contents`][check.invalid-indentation-multiple.num-indents-for-module-contents]
- [`check.invalid-indentation-multiple.num-indents-for-program-contents`][check.invalid-indentation-multiple.num-indents-for-program-contents]
- [`check.invalid-indentation-multiple.num-indents-for-select-contents`][check.invalid-indentation-multiple.num-indents-for-select-contents]
- [`check.invalid-indentation-multiple.num-indents-for-submodule-contents`][check.invalid-indentation-multiple.num-indents-for-submodule-contents]
- [`check.invalid-indentation-multiple.num-indents-for-line-continuation`][check.invalid-indentation-multiple.num-indents-for-line-continuation]
- [`check.invalid-indentation-multiple.num-indents-for-subroutine-contents`][check.invalid-indentation-multiple.num-indents-for-subroutine-contents]
- [`check.invalid-indentation-multiple.should-indent-associate-contents`][check.invalid-indentation-multiple.should-indent-associate-contents]
- [`check.invalid-indentation-multiple.should-indent-block-contents`][check.invalid-indentation-multiple.should-indent-block-contents]
- [`check.invalid-indentation-multiple.should-indent-derived-type-contents`][check.invalid-indentation-multiple.should-indent-derived-type-contents]
- [`check.invalid-indentation-multiple.should-indent-do-contents`][check.invalid-indentation-multiple.should-indent-do-contents]
- [`check.invalid-indentation-multiple.should-indent-function-contents`][check.invalid-indentation-multiple.should-indent-function-contents]
- [`check.invalid-indentation-multiple.should-indent-if-contents`][check.invalid-indentation-multiple.should-indent-if-contents]
- [`check.invalid-indentation-multiple.should-indent-interface-contents`][check.invalid-indentation-multiple.should-indent-interface-contents]
- [`check.invalid-indentation-multiple.should-indent-module-contents`][check.invalid-indentation-multiple.should-indent-module-contents]
- [`check.invalid-indentation-multiple.should-indent-program-contents`][check.invalid-indentation-multiple.should-indent-program-contents]
- [`check.invalid-indentation-multiple.should-indent-select-contents`][check.invalid-indentation-multiple.should-indent-select-contents]
- [`check.invalid-indentation-multiple.should-indent-submodule-contents`][check.invalid-indentation-multiple.should-indent-submodule-contents]
- [`check.invalid-indentation-multiple.should-indent-subroutine-contents`][check.invalid-indentation-multiple.should-indent-subroutine-contents]
- [`check.invalid-indentation-multiple.should-indent-after-line-continuation`][check.invalid-indentation-multiple.should-indent-after-line-continuation]


[check.indent-width]: ../settings.md#check_indent-width
[check.invalid-indentation-multiple.num-indents-for-associate-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-associate-contents
[check.invalid-indentation-multiple.num-indents-for-block-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-block-contents
[check.invalid-indentation-multiple.num-indents-for-derived-type-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-derived-type-contents
[check.invalid-indentation-multiple.num-indents-for-do-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-do-contents
[check.invalid-indentation-multiple.num-indents-for-function-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-function-contents
[check.invalid-indentation-multiple.num-indents-for-if-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-if-contents
[check.invalid-indentation-multiple.num-indents-for-interface-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-interface-contents
[check.invalid-indentation-multiple.num-indents-for-module-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-module-contents
[check.invalid-indentation-multiple.num-indents-for-program-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-program-contents
[check.invalid-indentation-multiple.num-indents-for-select-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-select-contents
[check.invalid-indentation-multiple.num-indents-for-submodule-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-submodule-contents
[check.invalid-indentation-multiple.num-indents-for-line-continuation]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-line-continuation
[check.invalid-indentation-multiple.num-indents-for-subroutine-contents]: ../settings.md#check_invalid-indentation-multiple_num-indents-for-subroutine-contents
[check.invalid-indentation-multiple.should-indent-associate-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-associate-contents
[check.invalid-indentation-multiple.should-indent-block-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-block-contents
[check.invalid-indentation-multiple.should-indent-derived-type-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-derived-type-contents
[check.invalid-indentation-multiple.should-indent-do-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-do-contents
[check.invalid-indentation-multiple.should-indent-function-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-function-contents
[check.invalid-indentation-multiple.should-indent-if-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-if-contents
[check.invalid-indentation-multiple.should-indent-interface-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-interface-contents
[check.invalid-indentation-multiple.should-indent-module-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-module-contents
[check.invalid-indentation-multiple.should-indent-program-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-program-contents
[check.invalid-indentation-multiple.should-indent-select-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-select-contents
[check.invalid-indentation-multiple.should-indent-submodule-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-submodule-contents
[check.invalid-indentation-multiple.should-indent-subroutine-contents]: ../settings.md#check_invalid-indentation-multiple_should-indent-subroutine-contents
[check.invalid-indentation-multiple.should-indent-after-line-continuation]: ../settings.md#check_invalid-indentation-multiple_should-indent-after-line-continuation

