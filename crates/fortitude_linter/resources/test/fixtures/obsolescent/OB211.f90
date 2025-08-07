module obsolete_openmp

    ! This is the correct way to get the OpenMP library
    use omp_lib

    ! Deprecated since OpenMP 6.0
    include "omp_lib.h"

end module obsolete_openmp
