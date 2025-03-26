# double-precision-literal (MOD002)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for floating point literals that using `d` or `D` as the exponentiation.

## Why is this bad?
Floating point literals using `d` or `D` as the exponentiation, such as `1.23d2`,
will be of the `double precision` kind. This is commonly assumed to be a 64-bit
float, but is not guaranteed to be so, and may vary depending on your system and
compiler arguments.

In modern Fortran, it is preferred to set the required precision using
'kinds'. For portability, it is recommended to use `real(dp)`, with `dp` set
in one of the following ways:

- `use, intrinsic :: iso_fortran_env, only: dp => real64`
- `integer, parameter :: dp = selected_real_kind(15, 307)`

For code that should be compatible with C, you should instead use
`real(c_double)`, which may be found in the intrinsic module `iso_c_binding`.
To ensure floating point literals match the kind of the variable they are assigned
to, it is recommended to use `e` or `E` for exponentiation and a kind suffix, so
`1.23d2` should be written as `1.23e2_dp`.

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained: Incorporating Fortran
  2018_, Oxford University Press, Appendix A 'Deprecated Features'
- [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)