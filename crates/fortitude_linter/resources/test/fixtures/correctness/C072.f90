program cases
  ! A char array outside a function or subroutine, no exception
  character (*) :: autochar_glob
contains
  subroutine char_input(autochar_in, autochar_inout, autochar_out, fixedchar)
    ! A char array with proper intent, no exception
    character(*), intent(in)       :: autochar_in
    ! A char array with disallowed intent, exception
    character(*), intent(inout)    :: autochar_inout
    ! A char array with disallowed intent, exception
    character(len=*), intent(out)  :: autochar_out
    ! A char array not passed as a parameter, no exception
    character(*)                   :: autochar_var
    ! A char array with fixed length, no exception
    character(len=10), intent(out) :: fixedchar
    ! A declaration with non-intent attribute, no exception
    character(len=*), parameter :: alt_attr = 'sample'
  end subroutine char_input
end program cases

module udtio
  implicit none (type, external)
  interface write(formatted)
    module procedure :: write_formatted
  end interface write(formatted)

contains
  ! This shouldn't raise because the standard mandates this signature for UDTIO
  subroutine write_formatted(self, unit, iotype, v_list, iostat, iomsg)
    class(demo_type), intent(in) :: self
    integer, intent(in) :: unit
    character(len=*), intent(in) :: iotype
    integer, intent(in) :: v_list(:)
    integer, intent(out) :: iostat
    character(len=*), intent(inout) :: iomsg
  end subroutine write_formatted
end module udtio
