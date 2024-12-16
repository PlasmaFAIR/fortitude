program test
  implicit none
  integer :: i, named_unit
  open(10, file="test.txt", action="read")
  read(10, *) i
  close(10)

  open(file="test_out.txt", action="write", unit=24)
  write (fmt=*, unit=24) i
  close(24)

  open(newunit=named_unit, file="test.txt", action="write")
  write(named_unit, *) "i =", i
  close(named_unit)
end program test
