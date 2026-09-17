program intrinsic_kind
  implicit none

  integer :: i
  real :: x, y
  complex :: z

  x = real(i)                 ! catch
  i = int(x)                  ! catch
  z = cmplx(x)                ! catch

  x = real(i, 8)              ! catch
  x = real(i, kind=8)         ! ignore
  i = int(x, 4)               ! catch
  i = int(x, kind=4)          ! ignore
  z = cmplx(x, y, 8)          ! catch
  z = cmplx(x, kind=8)        ! ignore
  z = cmplx(z, 8)             ! catch
  z = cmplx(x, y)             ! catch
end program intrinsic_kind
