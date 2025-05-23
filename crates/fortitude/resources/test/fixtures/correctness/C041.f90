program p
  implicit none
  integer :: file_unit
  integer :: stat

  open(123, file="test0", action="read")
  open(unit=234, file="test1")
  open(unit=345, file="test2", iostat=stat)
  open(unit=456, file="test3", access="append")
  open(unit=567, file="test4", action="write", iostat=stat, access="append")
  open(newunit=file_unit, file="test5", action="readwrite")

end program p
