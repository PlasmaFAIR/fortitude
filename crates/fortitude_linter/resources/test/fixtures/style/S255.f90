module test
  implicit none (external, type)
contains
  integer function capped_add(a, b)
    integer, intent(in) :: a, b
    if ((a + b) > 10) then
      capped_add = 10
      ! OK
      stop
    end if

    capped_add = a + b
  end function capped_add

  integer function capped_sub(a, b)
    integer, intent(in) :: a, b
    if ((a - b) < 0) then
      capped_sub = 0
      STOP
    else if ((a - b) > 10) then
      capped_sub = 10
      stop 3
    else
      capped_sub = a - b
    end if
  end function capped_sub

  integer function capped_mult(a, b)
    integer, intent(in) :: a, b
    if ((a * b) > 100) then
      capped_mult = 100
      error     stop 2
    else
      capped_mult = a * b
    end if
  end function capped_mult

  integer function capped_pow(a, b)
    integer, intent(in) :: a, b
    if ((a ** b) > 100) then
      capped_pow = 100
      if (b > 10) error stop 2 ! shouldn't trigger!
    else
      capped_pow = a ** b
    end if
  end function capped_pow
end module test
