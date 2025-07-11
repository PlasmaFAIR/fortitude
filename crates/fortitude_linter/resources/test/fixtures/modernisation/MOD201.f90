module obsolete_mpi

    ! This is the new MPI module that does more compile-time checks
    use mpi_f08

    ! Not technically deprecated, but discouraged
    use mpi

    ! This is fine
    use my_mpi_mod

    ! Discouraged since MPI 3.0, deprecated since MPI 4.1
    include "mpif.h"

end module obsolete_mpi
