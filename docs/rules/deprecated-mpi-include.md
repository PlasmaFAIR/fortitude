# deprecated-mpi-include (OB071)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for the use of the old deprecated `mpif.h` include.

## Why is this bad?
MPI used to use the `mpif.h` include file to define its subroutines and
constants, however that method has now been replaced by either the (not-recommended)
`mpi` module, or the newer `mpi_f08` module. The `mpif.h` include is
deprecated as of MPI 4.1.

The `mpi` module is a drop-in replacement for the `mpif.h` include (except
for its placement in the Fortran file), while the `mpi_f08` module requires
more code changes to use the MPI derived types for the constants.

## Examples

This MPI code using the `mpif.h` include

```f90
program demo_mpi

implicit none

include "mpif.h"

integer :: mpicomm
integer :: mpiint
integer myrank, nproc, ierr

mpicomm = MPI_COMM_WORLD
mpiint = MPI_INTEGER

call MPI_Init(ierr)
call MPI_Comm_rank(mpicomm, myrank, ierr)
call MPI_Comm_size(mpicomm, nproc, ierr)

call MPI_finalize(ierr)
end
```

can be converted to use the `mpi` module by simply changing the `include` statement
to a `use` statement

```f90
program demo_mpi
use mpi

implicit none

integer :: mpicomm
integer :: mpiint
integer myrank, nproc, ierr

mpicomm = MPI_COMM_WORLD
mpiint = MPI_INTEGER

call MPI_Init(ierr)
call MPI_Comm_rank(mpicomm, myrank, ierr)
call MPI_Comm_size(mpicomm, nproc, ierr)

call MPI_finalize(ierr)
```

or can be modernized to use the `mpi_f08` module

```f90
program demo_mpi_f08
use mpi_f08

implicit none

type(MPI_comm) :: mpicomm
type(MPI_datatype) :: mpiint
integer myrank, nproc, ierr

mpicomm = MPI_COMM_WORLD
mpiint = MPI_INTEGER

call MPI_Init(ierr)
call MPI_Comm_rank(mpicomm, myrank, ierr)
call MPI_Comm_size(mpicomm, nproc, ierr)

call MPI_finalize(ierr)
end
```

## References
- Message Passing Interface Forum, MPI: A Message-Passing Interface Standard, Jun. 2025.
  https://www.mpi-forum.org/docs/mpi-5.0/mpi50-report.pdf.
