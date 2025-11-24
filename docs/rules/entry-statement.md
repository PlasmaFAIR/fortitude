# entry-statement (OB021)
This rule is turned on by default.

## What it does
Checks for `entry` statements.

## Why is this bad?
`entry` statements are an obsolescent feature allowing more than entry point
into a procedure, enabling reuse of variables and executable
statements. However, they make the code much harder to follow and are prone
to bugs.

Multiple entry procedures can be replaced with modules to share data, and
private module procedures to reuse code.

## Notes
Entry statements were officially declared obsolescent in Fortran 2008, so
this rule only triggers if the target standard is Fortran 2008 or later.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
