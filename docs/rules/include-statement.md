# include-statement (M031)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for any include statements

## Why is this bad?
Include statements allow for pasting the contents of other files into
the current scope, which could be used for sharing COMMON blocks, procedures
or declaring variables. This can hide details from the programmer, increase
the maintenance burden and can be bug-prone. Avoided including files in
others and instead use modules.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix A
  'Deprecated Features'
- _Difference between INCLUDE and modules in Fortran_, 2013,
  https://stackoverflow.com/a/15668209