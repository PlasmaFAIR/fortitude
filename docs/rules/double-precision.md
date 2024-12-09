# double-precision (P011)
## What it does
Checks for use of `double precision` and `double complex` types.

## Why is this bad?
The `double precision` type does not guarantee a 64-bit floating point number
as one might expect, and instead is only required to have a higher decimal
precision than the default `real`, which may vary depending on your system
and can be modified by compiler arguments.

In modern Fortran, it is preferred to use `real` and `complex` and instead set
the required precision using 'kinds'. For portability, it is recommended to use
`real(dp)`, with `dp` set in one of the following ways:

- `use, intrinsic :: iso_fortran_env, only: dp => real64`
- `integer, parameter :: dp = selected_real_kind(15, 307)`

For code that should be compatible with C, you should instead use
`real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained: Incorporating Fortran
  2018_, Oxford University Press, Appendix A 'Deprecated Features'
- [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/en/learn/best_practices/floating_point/)