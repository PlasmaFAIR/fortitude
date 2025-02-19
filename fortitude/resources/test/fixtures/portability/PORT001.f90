program test
  implicit none
  integer :: i, named_unit

  write (6,*) "enter an integer"
  read (unit=5,fmt=*) i
  write(fmt=*, unit=6) "thanks"

  open(newunit=named_unit, file="test.txt", action="write")
  write(named_unit, *) "i =", i
  close(named_unit)
end program test
