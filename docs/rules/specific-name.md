# specific-name (OB031)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What does it do?
Checks for uses of the deprecated specific names of intrinsic functions.

## Why is this bad?
Specific names of intrinsic functions can be obscure and hinder readability of
the code. Fortran 90 made these specific names redundant and recommends the use
of the generic names for calling intrinsic functions.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'