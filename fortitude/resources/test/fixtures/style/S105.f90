program myprog
  implicit none
  integer :: a = 1
  integer, allocatable :: b(:)
  if(a ==1)then  ! A space should be added after if and before then
    a = 2
  else if  (a == 2)then ! A space should be removed after if and added before then
    a = 3
  else if &
    (a == 3) then ! This should be unchanged
    a = 4
  else if(a == 4)  then ! A space should be added after if and removed before then
    a = 5
  end if
  write (*,*) "a = ", a  ! This should be unchanged
  allocate (b(3))  ! This should be unchanged
  b = [1, 2, 3]
  write (*,*) "b = ", b  ! This should be unchanged
  deallocate (b)  ! This should be unchanged
  allocate(b(4))  ! A space should be added after allocate
  b = [1, 2, 3, 4]
  write(*,*) "b = ", b  ! A space should be added after write
  deallocate(b)  ! A space should be added after deallocate
end program myprog
