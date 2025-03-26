# pause-statement (OB051)
This rule is turned on by default.

## What it does
Checks for `pause` statements.

## Why is this bad?
`pause` statements were never properly standardised, doing different things
on different compilers, and were completely removed in Fortran 95. They can
usually be replaced with a simple call to `read(*,*)`

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
