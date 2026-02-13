# equivalence-statement (OB012)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for use of `equivalence` statements.

## Why is this bad?
Prior to Fortran 90, `equivalence` was a versatile and powerful statement,
but error-prone and easily abused. Fortran 90 introduced many safer features
which have made `equivalence` redundant, and Fortran 2018 officially made
the statement obsolescent.

Depending on its use case, `equivalence` statements should be replaced with:
- automatic arrays,
- allocatable arrays,
- pointers to reuse storage,
- pointers as aliases,
- or the `transfer` function for bit manipulation.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
