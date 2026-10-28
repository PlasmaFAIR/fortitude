! Expect indentation width of 3 spaces throughout.
! This test is simply to check that the correct spaces are highlighted
! in the diagnostics. More complex cases are tested in S105_large.f90
! and in the unit tests for incorrect-indentation.
module m
   implicit none (type, external)

private  ! Should be indented to 3 spaces

   contains  ! Should be dedented to 0 spaces

   subroutine s()
   integer :: i  ! Should be indented to 6 spaces
      do i = 1, 10
         print *, i
      end do
   end subroutine s

end module m
