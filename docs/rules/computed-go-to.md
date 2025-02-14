# computed-go-to (OB041)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for computed go to statements

## Why is this bad?
Computed go to statements are an obsolescent feature that allows selecting
the target of the jump from a list of possible targets using a variable.
They can be complicated to setup and hard to read, and should be replaced
by a SELECT CASE statement.


```f90
     target = degree +1
     GO TO (10, 20, 30, 40, 50) target
10   p = 1.0
     GO TO 100
20   p = x
     GO TO 100
30   p = 1.5*x**2 - 0.5
     GO TO 100
40   p = 2.5*x**3 - 1.5*x
     GO TO 100
50   p = 4.375*x**4 - 3.75*x**2 + 0.375
100
```

## Examples

Computing the Legendre polynomial of `degree`:

```f90
     target = degree +1
     GO TO (10, 20, 30, 40, 50) target
10   p = 1.0
     GO TO 100
20   p = x
     GO TO 100
30   p = 1.5*x**2 - 0.5
     GO TO 100
40   p = 2.5*x**3 - 1.5*x
     GO TO 100
50   p = 4.375*x**4 - 3.75*x**2 + 0.375
100
```
becomes:

```f90
SELECT CASE(degree)
case(0)
    p = 1.0
case(1)
    p = x
case(2)
    p = 1.5*x**2 - 0.5
case(3)
    p = 2.5*x**3 - 1.5*x
case(4)
    p = 4.375*x**4 - 3.75*x**2 + 0.375
END SELECT
```

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'