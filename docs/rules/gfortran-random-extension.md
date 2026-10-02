# gfortran-random-extension (PORT062)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What does it do?
Checks for gfortran random number extensions, srand and rand, and suggests
to replace them with fortran standard intrinsics random_seed and random_number
respectively.

## Why is this bad?
Using compiler specific instrinsics is inherently unportable as other users
will need to compile their code with the specific compiler. It can also make
code harder to understand if someone is not familiar with the extension. Using
Fortran standard implementations is preferred, and in some cases implement improved
algorithms.

## Example
``f90
call srand(seed)
``

``f90
x = rand(seed)
``

## Use instead
Prefer using Fortran standard intrinsics and libraries. In the case of random
number generation, use the `random_seed` and `random_number` intrinsics instead.
These are not interchangable replacements, as .e.g `random_number(x)` is a subroutine
that modifies `x` whereas `rand()` is a function that returns a single real.

## References
[GFortran docs for `srand`](https://gcc.gnu.org/onlinedocs/gfortran/SRAND.html)
[GFortran docs for `rand`](https://gcc.gnu.org/onlinedocs/gfortran/RAND.html)
[GFortran docs for `random_seed`](https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fSEED.html)
[GFortran docs for `random_number`](https://gcc.gnu.org/onlinedocs/gfortran/RANDOM_005fNUMBER.html)
