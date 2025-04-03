pure elemental logical function test(x, y) result(value)
  REAL(8) :: x, y
  logical :: value = (x == y)

contains

  real impure function internal_proc result(x)
    real(8) x = 1.0
  end function

  subroutine increment(i)
    integer :: i
    i = i + 1
  end subroutine

  real simple elemental function convert(a)
    type(foo) :: a
  end function convert

end function test
