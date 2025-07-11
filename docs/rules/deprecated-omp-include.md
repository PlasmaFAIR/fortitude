# deprecated-omp-include (OB211)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for the use of the deprecated `omp_lib.h` include.

## Why is this bad?
OpenMP deprecated the `omp_lib.h` include file in OpenMP 6.0. It is recommended
to switch to the `omp_lib` module

The `omp_lib` module should be a drop-in replacement for the `omp_lib.h` include (except
for its placement in the Fortran file).

## References
OpenMP Architecture Review Board, OpenMP Application Programming Interface 6.0,
Nov. 2024. Available: https://www.openmp.org/wp-content/uploads/OpenMP-API-Specification-6-0.pdf
