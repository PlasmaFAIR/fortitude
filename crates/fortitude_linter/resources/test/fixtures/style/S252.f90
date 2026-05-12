module test
  implicit none (external, type)
contains
  integer function capped_add(a, b)
    integer, intent(in) :: a, b
    if ((a + b) > 10) then
      capped_add = 10
      ! OK
      return
    end if

    capped_add = a + b
  end function capped_add

  integer function capped_sub(a, b)
    integer, intent(in) :: a, b
    if ((a - b) < 0) then
      capped_sub = 0
      return
    else if ((a - b) > 10) then
      capped_sub = 10
      return
    ELSEIF (.false.) then
      capped_sub = -1
      return
    else
      capped_sub = a - b
    end if
  end function capped_sub

  integer function capped_mult(a, b)
    integer, intent(in) :: a, b
    if ((a * b) > 100) then
      capped_mult = 100
      return
    else  ! This comment should be moved onto next line
      ! And some other bits that
      ! should also be unindented
 ! unindented comment shouldn't cause over-indentation
      capped_mult = a * b
    end if
  end function capped_mult

  integer function capped_div(a, b)
    integer, intent(in) :: a, b
    if ((a * b) > 100) then
      capped_div = 100
      stop                      ! shouldn't trigger!
    else
      capped_div = a * b
    end if
  end function capped_div

  integer function capped_pow(a, b)
    integer, intent(in) :: a, b
    if ((a ** b) > 100) then
      capped_pow = 100
      if (b > 10) return ! shouldn't trigger!
    else
      capped_pow = a ** b
    end if
  end function capped_pow

  integer function capped_double_add(a, b)
    integer, intent(in) :: a, b
    something: if ((a + b + b) > 100) then
      capped_double_add = 100
      return
    else
      ! Can't auto-fix due to label
      if (a == 4) exit something
      capped_double_add = a + b + b
    end if something

    capped_double_add = a + b + b
  end function capped_double_add
end module test
