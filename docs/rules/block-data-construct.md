# block-data-construct (OB013)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `block data` constructs.

## Why is this bad?
Fortran 90 introduced modules, which made `common` blocks redundant, and
with them the `block data` construct.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
