module test
  implicit none (external, type)
contains
  integer function capped_add(a, b)
    integer, intent(in) :: a, b
    integer :: i
    capped_add = 0
    do i = 1, a
      if ((i + b) > 10) then
        cycle
      end if
      capped_add = capped_add + b
    end do
  end function capped_add

  integer function capped_sub(a, b)
    integer, intent(in) :: a, b
    integer :: i
    capped_sub = 0
    do i = 1, a
      if ((i - b) < 0) then
        cycle
      else if ((i - b) > 10) then
        cycle
      else
        capped_sub = capped_sub - b
      end if
    end do
  end function capped_sub
      

  integer function capped_mult(a, b)
    integer, intent(in) :: a, b
    integer :: i
    capped_mult = 1
    label: do i = 1, a
      if ((a * b) > 100) then
        cycle label
      else
        capped_mult = capped_mult * b
      end if
    end do label
  end function capped_mult
end module test
