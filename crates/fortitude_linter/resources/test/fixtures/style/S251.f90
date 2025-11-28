module test
  implicit none (external, type)
contains
  integer function capped_add(a, b)
    integer, intent(in) :: a, b
    if ((a + b) > 10) then
      capped_add = 10
      return
    end if
    capped_add = a + b
    return
  end function capped_add

  integer function capped_sub(a, b)
    integer, intent(in) :: a, b
    if ((a - b) < 0) then
      capped_sub = 0
      return
    end if
    capped_sub = a - b
    return
    ! but with comments

    ! and whitespace after
  end function capped_sub

end module test
