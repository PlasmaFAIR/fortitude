# forall-statement (OB071)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for `forall` statements.

## Why is this bad?
The F2018 standard made `forall` statements obsolescent in favour of `do
concurrent`. They were orginally added with the intention of parallelising
loops across multiple processors, however, they turned out to have too many
restrictions for compilers to be able to take full advantage of them.

Instead, the `do concurrent` statement was introduced, which solved many of
these difficulties (although not without its own issues, see [1]), along
with the use of pointer rank remapping.

## Example
```f90
forall (i=1:N)
  b(i) = a(i) * c(i)
end forall
```

Use instead:
```f90
do concurrent (i=1:N)
  b(i) = a(i) * c(i)
end do concurrent
```

## References
- [1]: [`DO CONCURRENT` isn’t necessarily concurrent](https://flang.llvm.org/docs/DoConcurrent.html)
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
